use crate::{check_cancel,err,safe_path,Result};
use std::{fs,io::{Read,Write},path::Path,sync::atomic::AtomicBool};
/// Extraction is staged; links are created only after checking their resolved target stays inside.
pub fn extract_jdk(archive: &Path, stage: &Path, cancel: &AtomicBool) -> Result<()> {
    let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(
        fs::File::open(archive).map_err(err)?,
    ));
    let mut links = vec![];
    let mut total = 0u64;
    for (index, entry) in tar.entries().map_err(err)?.enumerate() {
        check_cancel(cancel)?;
        if index > 100_000 {
            return Err("Java archive contains too many files".into());
        }
        let mut entry = entry.map_err(err)?;
        let relative = entry
            .path()
            .map_err(err)?
            .to_str()
            .ok_or("Invalid Java filename")?
            .trim_end_matches('/')
            .to_string();
        let path = safe_path(stage, &relative)?;
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            fs::create_dir_all(path).map_err(err)?;
            continue;
        }
        if kind.is_symlink() || kind.is_hard_link() {
            links.push((
                path,
                entry
                    .link_name()
                    .map_err(err)?
                    .ok_or("Invalid Java link")?
                    .into_owned(),
                kind.is_hard_link(),
            ));
            continue;
        }
        if !kind.is_file() {
            return Err("Java archive contains an unsupported file type".into());
        }
        total += entry.size();
        if total > 5 * 1024 * 1024 * 1024 {
            return Err("Java extracted size exceeds the limit".into());
        }
        fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
        let mut output = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(err)?;
        let mut buf = [0u8; 65536];
        loop {
            check_cancel(cancel)?;
            let n = entry.read(&mut buf).map_err(err)?;
            if n == 0 {
                break;
            }
            output.write_all(&buf[..n]).map_err(err)?;
        }
        launcher_platform::set_extracted_permissions(&path,entry.header().mode().map_err(err)?)?;
    }
    let canonical = stage.canonicalize().map_err(err)?;
    for (path, target, hard) in links {
        check_cancel(cancel)?;
        if target.is_absolute() {
            return Err("Java archive link escapes the extraction directory".into());
        }
        let resolved = if hard {
            stage.join(&target)
        } else {
            path.parent().unwrap().join(&target)
        }
        .canonicalize()
        .map_err(err)?;
        if !resolved.starts_with(&canonical) {
            return Err("Java archive link escapes the extraction directory".into());
        }
        fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
        if hard {
            fs::hard_link(resolved, path).map_err(err)?;
        } else {
            launcher_platform::create_runtime_symlink(&target,&path)?;
        }
    }
    Ok(())
}
pub fn extract(archive:&Path,stage:&Path,format:&str,cancel:&AtomicBool)->Result<()> {
    match format {"tar.gz"=>extract_jdk(archive,stage,cancel),"zip"=>extract_zip(archive,stage,cancel),_=>Err("Unsupported Java archive format".into())}
}
fn extract_zip(archive:&Path,stage:&Path,cancel:&AtomicBool)->Result<()> {
    let mut zip=zip::ZipArchive::new(fs::File::open(archive).map_err(err)?).map_err(err)?;
    if zip.len()>100_000{return Err("Java archive contains too many files".into())}
    let mut total=0u64;
    for i in 0..zip.len() {
        check_cancel(cancel)?;
        let mut entry=zip.by_index(i).map_err(err)?;
        if entry.is_symlink(){return Err("Java archive contains an unsupported file type".into())}
        let path=safe_path(stage,entry.name().trim_end_matches('/'))?;
        if entry.is_dir(){fs::create_dir_all(path).map_err(err)?;continue}
        total=total.checked_add(entry.size()).ok_or("Java extracted size exceeds the limit")?;
        if total>5*1024*1024*1024{return Err("Java extracted size exceeds the limit".into())}
        fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
        let mut output=fs::OpenOptions::new().create_new(true).write(true).open(path).map_err(err)?;
        let mut buffer=[0u8;65536];let mut count=0;
        loop {check_cancel(cancel)?;let n=entry.read(&mut buffer).map_err(err)?;if n==0{break}count+=n as u64;if count>entry.size(){return Err("Java extracted size exceeds the limit".into())}output.write_all(&buffer[..n]).map_err(err)?;}
        if count!=entry.size(){return Err("Java archive size mismatch".into())}
    }Ok(())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn zip_runtime_is_portable_and_rejects_escape_and_overwrite() {
        for (names,valid) in [(vec!["jdk/bin/java.exe","jdk/lib/modules"],true),(vec!["../escape"],false),(vec!["C:/evil.exe"],false)] {
            let root=tempfile::tempdir().unwrap();let archive=root.path().join("java.zip");
            let mut zip=zip::ZipWriter::new(fs::File::create(&archive).unwrap());
            for name in names {zip.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();zip.write_all(b"fixture").unwrap();}zip.finish().unwrap();
            let stage=root.path().join("stage");fs::create_dir(&stage).unwrap();
            assert_eq!(extract(&archive,&stage,"zip",&AtomicBool::new(false)).is_ok(),valid);
            assert!(!root.path().join("escape").exists());
            if valid {
                let executable=stage.join("jdk/bin/java.exe");
                fs::write(&executable,b"existing runtime").unwrap();
                assert!(extract(&archive,&stage,"zip",&AtomicBool::new(false)).is_err());
                assert_eq!(fs::read(executable).unwrap(),b"existing runtime");
            }
        }
    }
}
