//! Host executable replacement. No release discovery, download or UI policy.
use std::{path::{Path,PathBuf}, process::Command, time::{Duration,Instant}, fs};

pub fn target() -> Result<(&'static str,PathBuf),String> {
    if cfg!(debug_assertions) { return Err("Self-update is unavailable in development builds".into()); }
    #[cfg(all(target_os="linux",target_arch="x86_64"))]
    {
        let path=std::env::var_os("APPIMAGE").ok_or("Self-update requires an AppImage")?;
        Ok(("linux-x86_64", fs::canonicalize(path).map_err(|e|e.to_string())?))
    }
    #[cfg(all(target_os="windows",target_arch="x86_64"))]
    { Ok(("win-x86_64",std::env::current_exe().map_err(|e|e.to_string())?)) }
    #[cfg(not(all(any(target_os="linux",target_os="windows"),target_arch="x86_64")))]
    { Err("Launcher updates are unavailable for this platform".into()) }
}

/// The caller must exit only after this returns, releasing its session lock.
/// Both paths must be trusted absolute paths, on the same filesystem.
pub fn replace_after_exit(destination:&Path, staged:&Path) -> Result<(),String> {
    for path in [destination,staged] {
        let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
        if !path.is_absolute() || !meta.is_file() || meta.file_type().is_symlink() { return Err("Invalid executable update path".into()); }
    }
    let directory=staged.parent().ok_or("Invalid staging directory")?;
    let ready=directory.join("ready");
    let log=directory.join("apply.log");
    let backup=PathBuf::from(format!("{}.previous",destination.display()));
    if let Ok(meta)=fs::symlink_metadata(&backup) {
        if !meta.is_file() || meta.file_type().is_symlink() { return Err("Invalid update backup path".into()); }
    }
    let mut command=helper(destination,staged,&backup,&ready)?;
    command.stdout(fs::File::create(&log).map_err(|e|e.to_string())?).stderr(fs::File::options().append(true).open(&log).map_err(|e|e.to_string())?);
    let mut child=command.spawn().map_err(|e|e.to_string())?;
    let started=Instant::now();
    while started.elapsed()<Duration::from_secs(10) {
        if ready.is_file() { return Ok(()); }
        if child.try_wait().map_err(|e|e.to_string())?.is_some() { return Err(format!("Update helper failed; see {}",log.display())); }
        std::thread::sleep(Duration::from_millis(50));
    }
    let _=child.kill(); let _=child.wait();
    Err("Update helper did not become ready".into())
}

#[cfg(target_os="linux")]
fn helper(destination:&Path,staged:&Path,backup:&Path,ready:&Path)->Result<Command,String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(staged,fs::Permissions::from_mode(0o755)).map_err(|e|e.to_string())?;
    let script=staged.parent().unwrap().join("apply.sh");
    fs::write(&script,include_str!("linux_update.sh")).map_err(|e|e.to_string())?;
    let mut c=Command::new("/bin/sh");
    c.arg(script).arg(std::process::id().to_string()).arg(destination).arg(staged).arg(backup).arg(ready);
    super::clean_child_environment(&mut c);
    Ok(c)
}
#[cfg(target_os="windows")]
fn helper(destination:&Path,staged:&Path,backup:&Path,ready:&Path)->Result<Command,String> {
    use base64::Engine;
    use std::os::windows::process::CommandExt;
    let quote=|p:&Path| format!("'{}'",p.to_string_lossy().replace('\'',"''"));
    let script=format!("$oldPid={};$destination={};$staged={};$backup={};$ready={};\n{}",std::process::id(),quote(destination),quote(staged),quote(backup),quote(ready),include_str!("windows/update.ps1"));
    let encoded=base64::engine::general_purpose::STANDARD.encode(script.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>());
    let mut c=Command::new("powershell.exe");
    c.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-EncodedCommand",&encoded]).creation_flags(0x08000000);
    Ok(c)
}
#[cfg(not(any(target_os="windows",target_os="linux")))]
fn helper(_: &Path,_:&Path,_:&Path,_:&Path)->Result<Command,String>{Err("Unsupported update platform".into())}

#[cfg(all(test,target_os="linux"))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn folder()->PathBuf{
        let p=std::env::temp_dir().join(format!("update test ' {}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&p).unwrap();p
    }
    #[test]
    fn replacement_preserves_name_backup_working_directory_and_steam_environment(){
        let root=folder();let dest=root.join("custom ' name.AppImage");let staged=root.join("new.AppImage");let backup=root.join("custom ' name.AppImage.previous");
        fs::write(&dest,b"old binary").unwrap();fs::write(&backup,b"older binary").unwrap();
        fs::write(&staged,b"#!/bin/sh\nprintf '%s' \"$SteamGameId\" > result\n[ -z \"${APPIMAGE:-}\" ]\n").unwrap();
        fs::set_permissions(&staged,fs::Permissions::from_mode(0o755)).unwrap();
        let status=Command::new("/bin/sh").arg("-c").arg(include_str!("linux_update.sh")).arg("update").arg("2147483647").arg(&dest).arg(&staged).arg(&backup).arg(root.join("ready")).env("SteamGameId","123456").env("APPIMAGE","stale").status().unwrap();
        assert!(status.success());assert_eq!(fs::read(&backup).unwrap(),b"old binary");assert_eq!(fs::read(root.join("result")).unwrap(),b"123456");assert!(dest.is_file());assert!(!staged.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_move_restores_original(){
        let root=folder();let dest=root.join("launcher.AppImage");let backup=root.join("launcher.AppImage.previous");
        fs::write(&dest,b"original").unwrap();
        let output=Command::new("/bin/sh").arg("-c").arg(include_str!("linux_update.sh")).arg("update").arg("2147483647").arg(&dest).arg(root.join("missing")).arg(&backup).arg(root.join("ready")).output().unwrap();
        assert!(!output.status.success());assert_eq!(fs::read(&dest).unwrap(),b"original");assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_links_before_spawning(){
        let root=folder();let dest=root.join("launcher.AppImage");let staged=root.join("new");
        fs::write(&dest,b"original").unwrap();std::os::unix::fs::symlink(&dest,&staged).unwrap();
        assert!(replace_after_exit(&dest,&staged).is_err());assert_eq!(fs::read(&dest).unwrap(),b"original");fs::remove_dir_all(root).unwrap();
    }
}
