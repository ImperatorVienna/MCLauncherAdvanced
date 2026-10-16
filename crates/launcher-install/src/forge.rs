//! Forge-family adapter: delegate patch processors to the verified official installer.
//! Only validated runtime artifacts leave the temporary installation root.
use crate::{check_cancel, download::{self, File}, err, java, minecraft, phase, safe_path, Report, Result};
use launcher_core::ModLoader;
use serde_json::{json, Value};
use sha1::Digest;
use std::{fs, io::Read, path::Path, process::{Command, Stdio}, sync::atomic::AtomicBool, time::{Duration, Instant}};
use crate::loaders::LoaderVersion;

pub fn repository(kind: &str) -> Result<&'static str> {
    match kind {
        "forge" => Ok("https://maven.minecraftforge.net/net/minecraftforge/forge"),
        "neoforge" => Ok("https://maven.neoforged.net/releases/net/neoforged/neoforge"),
        _ => Err("Invalid mod loader selection".into()),
    }
}
pub fn supported_game(game: &str) -> bool {
    let Ok(parts) = game.split('.').map(str::parse::<u32>).collect::<std::result::Result<Vec<_>,_>>() else { return false; };
    (2..=3).contains(&parts.len()) && (parts[0] == 1 && parts[1] >= 13 || parts[0] >= 26)
}
fn neo_game(version: &str) -> Option<String> {
    // Snapshot builds encode an additional game suffix and are not release matches.
    if version.contains('+') { return None; }
    let parts: Vec<u32> = version.split('-').next()?.split('.').map(str::parse).collect::<std::result::Result<_, _>>().ok()?;
    match parts.as_slice() {
        [major @ 20..=25, minor, _] => Some(if *minor == 0 { format!("1.{major}") } else { format!("1.{major}.{minor}") }),
        [major, minor, patch, _] if *major >= 26 => Some(if *patch == 0 { format!("{major}.{minor}") } else { format!("{major}.{minor}.{patch}") }),
        _ => None,
    }
}
pub fn parse_catalog(kind: &str, game: &str, xml: &str) -> Result<Vec<LoaderVersion>> {
    repository(kind)?;
    if !supported_game(game) { return Ok(vec![]); }
    if !xml.contains("<metadata") || !xml.contains("</metadata>") { return Err("Invalid mod loader catalog".into()); }
    let mut out = vec![];
    for (offset, _) in xml.match_indices("<version>") {
        let tail = &xml[offset..];
        let value = tail.strip_prefix("<version>").unwrap().split_once("</version>").ok_or("Invalid mod loader catalog")?.0.trim();
        let version = if kind == "forge" { value.strip_prefix(&format!("{game}-")) } else { (neo_game(value).as_deref() == Some(game)).then_some(value) };
        if let Some(version) = version {
            ModLoader { kind: kind.into(), version: version.into() }.validate()?;
            out.push(LoaderVersion {version: version.into(), stable: !version.contains('-')});
        }
    }
    crate::loaders::sort_versions(&mut out);
    Ok(out)
}
pub fn catalog(kind: &str, game: &str) -> Result<Vec<LoaderVersion>> {
    if !supported_game(game) { return Ok(vec![]); }
    let xml = String::from_utf8(download::bytes(&download::client()?, &format!("{}/maven-metadata.xml", repository(kind)?), 16 * 1024 * 1024)?).map_err(err)?;
    parse_catalog(kind, game, &xml)
}
fn archive_json(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> Result<Value> {
    let entry = archive.by_name(name).map_err(err)?;
    if entry.size() > 8 * 1024 * 1024 || entry.is_symlink() { return Err("Invalid mod loader profile".into()); }
    let mut data = vec![];
    entry.take(8 * 1024 * 1024 + 1).read_to_end(&mut data).map_err(err)?;
    serde_json::from_slice(&data).map_err(err)
}
fn installer_java(selected: Option<&Path>, major: u32, cancel: &AtomicBool) -> Result<java::Java> {
    if let Some(path) = selected {
        return java::validate_selected(path, major, cancel);
    }
    Err(format!("Mod loader installation requires a selected Java {major} or newer; select or download Java in instance management and retry"))
}
fn run_installer(java: &Path, jar: &Path, stage: &Path, log: &Path, cancel: &AtomicBool) -> Result<()> {
    let output = fs::File::create(log).map_err(err)?;
    let mut command = Command::new(java);
    command.args(["-Djava.awt.headless=true", "-jar"]).arg(launcher_platform::child_path(jar)).arg("--installClient").arg(launcher_platform::child_path(stage))
        .current_dir(stage).stdin(Stdio::null()).stdout(output.try_clone().map_err(err)?).stderr(output);
    // The official processors and any children share a cancellable process group.
    let mut child = launcher_platform::spawn_installer(&mut command)?;
    let started = Instant::now();
    loop {
        let stop = check_cancel(cancel).err().or_else(|| (started.elapsed() > Duration::from_secs(20 * 60)).then(|| "Mod loader installer timed out".into()))
            .or_else(|| fs::metadata(log).ok().filter(|m| m.len() > 32 * 1024 * 1024).map(|_| "Mod loader installer log exceeds the size limit".into()));
        if let Some(reason) = stop {
            launcher_platform::kill_installer(&mut child); return Err(reason);
        }
        match child.try_wait() {
            Ok(Some(status)) => return if status.success() { Ok(()) } else { Err(format!("Official mod loader installer failed; see log: {}", log.display())) },
            Ok(None) => std::thread::sleep(Duration::from_millis(100)),
            Err(e) => { launcher_platform::kill_installer(&mut child); return Err(err(e)); }
        }
    }
}
fn file_digest(path: &Path, cancel: &AtomicBool) -> Result<(String, u64)> {
    let mut file = fs::File::open(path).map_err(err)?;
    let size = file.metadata().map_err(err)?.len();
    if size == 0 || size > 4 * 1024 * 1024 * 1024 { return Err("Invalid mod loader artifact size".into()); }
    let mut hash = sha1::Sha1::new();
    let mut buffer = [0u8; 65536];
    loop {
        check_cancel(cancel)?;
        let n = file.read(&mut buffer).map_err(err)?;
        if n == 0 { break; }
        hash.update(&buffer[..n]);
    }
    Ok((format!("{:x}",hash.finalize()),size))
}
fn copy_verified(source: &Path, root: &Path, file: &File, cancel: &AtomicBool) -> Result<()> {
    if !download::valid_file(source, file, cancel)? { return Err(format!("Mod loader artifact verification failed: {}", file.path)); }
    let destination = safe_path(root, &file.path)?;
    if download::valid_file(&destination, file, cancel)? { return Ok(()); }
    fs::create_dir_all(destination.parent().unwrap()).map_err(err)?;
    let temp = tempfile::NamedTempFile::new_in(destination.parent().unwrap()).map_err(err)?;
    fs::copy(source, temp.path()).map_err(err)?;
    temp.as_file().sync_all().map_err(err)?;
    check_cancel(cancel)?;
    temp.persist(destination).map_err(err)?;
    Ok(())
}
// FML can discover generated game jars from libraryDirectory without placing
// them on the Java classpath. Preserve the installer's complete library output,
// recording auxiliary files separately from launch libraries.
fn import_auxiliary(stage: &Path, root: &Path, profile: &Value, url: &str, cancel: &AtomicBool) -> Result<Vec<File>> {
    let listed: std::collections::HashSet<String> = profile["libraries"].as_array().ok_or("Invalid mod loader libraries")?
        .iter().filter_map(|lib| lib["downloads"]["artifact"]["path"].as_str().map(|p| format!("libraries/{p}"))).collect();
    let mut pending = vec![stage.join("libraries")];
    let mut files = vec![];
    while let Some(directory) = pending.pop() {
        check_cancel(cancel)?;
        for entry in fs::read_dir(directory).map_err(err)? {
            let entry = entry.map_err(err)?;
            let kind = entry.file_type().map_err(err)?;
            let path = entry.path();
            if kind.is_symlink() { return Err("Invalid installer output symlink".into()); }
            if kind.is_dir() { pending.push(path); continue; }
            if !kind.is_file() { return Err("Invalid installer output file".into()); }
            let relative = path.strip_prefix(stage).map_err(err)?.to_str().ok_or("Invalid installer output path")?.to_owned();
            if listed.contains(&relative) { continue; }
            let source = safe_path(stage, &relative)?;
            let (hash, size) = file_digest(&source, cancel)?;
            let file = File {path:relative,url:url.into(),hash,size};
            file.validate()?;
            copy_verified(&source, root, &file, cancel)?;
            files.push(file);
        }
    }
    files.sort_by(|a,b| a.path.cmp(&b.path));
    Ok(files)
}
pub fn prepare(root: &Path, game: &str, directory: &str, loader: &ModLoader, vanilla: &Value, selected_java: Option<&Path>, cancel: &AtomicBool, report: &Report<'_>) -> Result<Value> {
    loader.validate()?;
    if !catalog(&loader.kind, game)?.iter().any(|v| v.version == loader.version) { return Err("The selected mod loader does not support this game version".into()); }
    let runtime = installer_java(selected_java, minecraft::java_major(vanilla)?, cancel)?;
    let coordinate = if loader.kind == "forge" {format!("{game}-{}",loader.version)} else {loader.version.clone()};
    let artifact_name = if loader.kind == "forge" {"forge"} else {"neoforge"};
    let url = format!("{}/{coordinate}/{artifact_name}-{coordinate}-installer.jar", repository(&loader.kind)?);
    let client = download::client()?;
    phase(report, "Downloading and verifying mod loader");
    let hash = String::from_utf8(download::bytes(&client, &format!("{url}.sha1"), 1024)?).map_err(err)?.split_whitespace().next().ok_or("Missing mod loader checksum")?.to_owned();
    let bytes = download::bytes(&client, &url, 64 * 1024 * 1024)?;
    if !download::hash_ok(&bytes, &hash) { return Err("Mod loader installer checksum mismatch".into()); }
    fs::create_dir_all(root).map_err(err)?;
    let stage = tempfile::Builder::new().prefix(".loader-stage-").tempdir_in(root).map_err(err)?;
    let jar = stage.path().join("installer.jar");
    fs::write(&jar, bytes).map_err(err)?;
    let mut archive = zip::ZipArchive::new(fs::File::open(&jar).map_err(err)?).map_err(err)?;
    let install = archive_json(&mut archive, "install_profile.json")?;
    let mut profile = archive_json(&mut archive, "version.json")?;
    if install["spec"] != 1 || install["minecraft"] != game || profile["inheritsFrom"] != game { return Err("Unsupported official mod loader installer format".into()); }
    // Validate the launch adapter before running potentially expensive processors.
    crate::loaders::merge(&mut vanilla.clone(), &profile, game)?;
    crate::atomic_json(&stage.path().join("launcher_profiles.json"), &json!({"profiles":{}}))?;
    let vanilla_dir = safe_path(stage.path(), &format!("versions/{game}"))?;
    fs::create_dir_all(&vanilla_dir).map_err(err)?;
    crate::atomic_json(&vanilla_dir.join(format!("{game}.json")), vanilla)?;
    let client_info = &vanilla["downloads"]["client"];
    let client_file = File {path:format!("versions/{game}/{game}.jar"),url:client_info["url"].as_str().ok_or("Missing client URL")?.into(),hash:client_info["sha1"].as_str().ok_or("Missing client checksum")?.into(),size:client_info["size"].as_u64().ok_or("Missing client size")?};
    download::download_all(&client, stage.path(), &[client_file], cancel, report, "Downloading and verifying game files")?;
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(err)?.as_nanos();
    let instance = minecraft::instance_directory(root, directory)?;
    let log = safe_path(&instance, &format!("logs/{}-{coordinate}-installer-{stamp}.log", loader.kind))?;
    fs::create_dir_all(log.parent().unwrap()).map_err(err)?;
    phase(report, "Running official mod loader installer");
    run_installer(&runtime.path, &jar, stage.path(), &log, cancel)?;
    phase(report, "Verifying generated mod loader files");
    let kernel = launcher_platform::kernel_version().unwrap_or_default();
    for library in profile["libraries"].as_array_mut().ok_or("Invalid mod loader libraries")? {
        if !minecraft::allowed(library, kernel.trim())? { continue; }
        check_cancel(cancel)?;
        let artifact = &mut library["downloads"]["artifact"];
        let relative = format!("libraries/{}", artifact["path"].as_str().ok_or("Invalid mod loader library path")?);
        let source = safe_path(stage.path(), &relative)?;
        let generated = artifact["url"].as_str() == Some("");
        let (actual_hash, size) = file_digest(&source, cancel)?;
        // Processor outputs can omit their checksum in the profile. The verified
        // installer validates its processor outputs; record their resulting hash.
        if !generated || artifact["sha1"].as_str().is_some_and(|s| !s.is_empty()) {
            if artifact["sha1"].as_str() != Some(&actual_hash) { return Err(format!("Mod loader artifact verification failed: {relative}")); }
        }
        if !generated && artifact["size"].as_u64() != Some(size) { return Err(format!("Mod loader artifact verification failed: {relative}")); }
        artifact["sha1"] = json!(actual_hash);
        artifact["size"] = json!(size);
        // Generated files are recreated by prepare on repair, never fetched as
        // standalone artifacts. The source installer URL records their provenance.
        if generated { artifact["url"] = json!(url); }
        let file = File {path:relative,url:artifact["url"].as_str().ok_or("Invalid mod loader repository")?.into(),hash:actual_hash,size:size};
        file.validate()?;
        copy_verified(&source, root, &file, cancel)?;
    }
    let auxiliary = import_auxiliary(stage.path(), root, &profile, &url, cancel)?;
    profile["launcherAuxiliaryFiles"] = serde_json::to_value(auxiliary).map_err(err)?;
    check_cancel(cancel)?;
    Ok(profile)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auxiliary_outputs_are_preserved_without_becoming_classpath_libraries() {
        let stage=tempfile::tempdir().unwrap(); let root=tempfile::tempdir().unwrap();
        let relative="libraries/net/neoforged/minecraft-client-patched/test/patched.jar";
        let source=stage.path().join(relative); fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(&source,b"patched game").unwrap();
        let profile=json!({"libraries":[]});
        let files=import_auxiliary(stage.path(),root.path(),&profile,"https://maven.neoforged.net/installer.jar",&AtomicBool::new(false)).unwrap();
        assert_eq!(files.len(),1); assert_eq!(files[0].path,relative);
        assert!(download::valid_file(&root.path().join(relative),&files[0],&AtomicBool::new(false)).unwrap());
        assert!(profile["libraries"].as_array().unwrap().is_empty());
        fs::remove_file(root.path().join(relative)).unwrap();
        assert!(!download::valid_file(&root.path().join(relative),&files[0],&AtomicBool::new(false)).unwrap());
    }
    #[test]
    #[ignore = "Uses isolated official installer diagnostic output"]
    fn real_neoforge_auxiliary_outputs() {
        let diagnostic=std::env::var("NEOFORGE_DIAGNOSTIC_ROOT").expect("Set NEOFORGE_DIAGNOSTIC_ROOT to the isolated official installation");
        let stage=Path::new(&diagnostic);
        let root=tempfile::tempdir().unwrap();
        let profile:Value=serde_json::from_slice(&fs::read(stage.join("versions/neoforge-26.3.0.77/neoforge-26.3.0.77.json")).unwrap()).unwrap();
        let files=import_auxiliary(stage,root.path(),&profile,"https://maven.neoforged.net/installer.jar",&AtomicBool::new(false)).unwrap();
        assert!(files.iter().any(|file| file.path.contains("minecraft-client-patched-26.3.0.77.jar")));
        assert!(files.iter().any(|file| file.path.contains("neoforge-26.3.0.77-universal.jar")));
    }
    #[test] fn catalog_matches_exact_game_and_sorts_releases() {
        let xml="<metadata><versions><version>1.21.1-52.0.2</version><version>1.21-51.0.1</version><version>1.21.1-52.0.10</version></versions></metadata>";
        let versions=parse_catalog("forge","1.21.1",xml).unwrap();
        assert_eq!(versions.iter().map(|v|v.version.as_str()).collect::<Vec<_>>(),["52.0.10","52.0.2"]);
        assert!(parse_catalog("forge","1.12.2",xml).unwrap().is_empty());
        assert_eq!(neo_game("21.0.167"),Some("1.21".into()));
        assert_eq!(neo_game("21.1.172"),Some("1.21.1".into()));
        assert_eq!(neo_game("26.3.0.77"),Some("26.3".into()));
        assert_eq!(neo_game("26.1.2.117"),Some("26.1.2".into()));
        assert_eq!(neo_game("26.1.0.0-alpha.1+snapshot-1"),None);
    }
    #[test] fn installer_requires_explicit_java_even_when_system_java_exists() {
        let error=installer_java(None,21,&AtomicBool::new(false)).unwrap_err();
        assert!(error.contains("requires a selected Java 21"));
    }
    #[test] fn corrupt_output_cannot_replace_cache() {
        let root=tempfile::tempdir().unwrap();let source=root.path().join("source");fs::write(&source,b"wrong").unwrap();
        let file=File {path:"libraries/test.jar".into(),url:"https://maven.minecraftforge.net/test".into(),hash:"a9993e364706816aba3e25717850c26c9cd0d89d".into(),size:3};
        assert!(copy_verified(&source,root.path(),&file,&AtomicBool::new(false)).is_err());
        assert!(!root.path().join(&file.path).exists());
    }
    #[test]
    #[cfg(unix)]
    fn cancellation_stops_official_installer_process() {
        use std::os::unix::fs::PermissionsExt;
        use std::sync::atomic::Ordering;
        let root=tempfile::tempdir().unwrap();
        let executable=root.path().join("fake-java");
        fs::write(&executable,b"#!/bin/sh\nsleep 60\n").unwrap();
        fs::set_permissions(&executable,fs::Permissions::from_mode(0o755)).unwrap();
        let cancel=AtomicBool::new(false);
        std::thread::scope(|scope| {
            scope.spawn(|| {std::thread::sleep(Duration::from_millis(100));cancel.store(true,Ordering::SeqCst);});
            let started=Instant::now();
            let result=run_installer(&executable,&root.path().join("installer.jar"),root.path(),&root.path().join("log"),&cancel);
            assert_eq!(result.unwrap_err(),"Task cancelled");
            assert!(started.elapsed()<Duration::from_secs(5));
        });
    }
    #[test]
    #[ignore = "Runs official installers and downloads dependencies in a temporary root"]
    fn official_forge_family_installers() {
        let root=tempfile::tempdir().unwrap();
        let versions=minecraft::catalog().unwrap();
        for (game, kind, version, java) in [
            ("1.21.1","forge","52.1.0","/usr/lib/jvm/java-21-openjdk/bin/java"),
            ("1.21.1","neoforge","21.1.172","/usr/lib/jvm/java-21-openjdk/bin/java"),
            ("26.3","forge","66.0.11","/usr/lib/jvm/java-27-openjdk/bin/java"),
            ("26.3","neoforge","26.3.0.77","/usr/lib/jvm/java-27-openjdk/bin/java"),
        ] {
            let metadata=versions.iter().find(|v|v.id==game).unwrap();
            let raw=download::bytes(&download::client().unwrap(),&metadata.url,8*1024*1024).unwrap();
            assert!(download::hash_ok(&raw,&metadata.sha1));
            let vanilla:Value=serde_json::from_slice(&raw).unwrap();
            let loader=ModLoader{kind:kind.into(),version:version.into()};
            let profile=prepare(root.path(),game,&format!("{game}-{kind}"),&loader,&vanilla,Some(Path::new(java)),&AtomicBool::new(false),&|p| eprintln!("{kind} {game}: {}",p.phase)).unwrap();
            let mut merged=vanilla.clone();
            crate::loaders::merge(&mut merged,&profile,game).unwrap();
            let plan=minecraft::plan(&merged,&json!({"objects":{}}),"6.1").unwrap();
            for file in plan.files.iter().filter(|f| f.path.starts_with("libraries/")) {
                // Vanilla-only dependencies are fetched by the ordinary game installer.
                if profile["libraries"].as_array().unwrap().iter().any(|l| format!("libraries/{}",l["downloads"]["artifact"]["path"].as_str().unwrap_or(""))==file.path) {
                    assert!(download::valid_file(&safe_path(root.path(),&file.path).unwrap(),file,&AtomicBool::new(false)).unwrap());
                }
            }
        }
    }
}
