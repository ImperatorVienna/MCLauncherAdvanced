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
pub fn replace_after_exit(current:&Path, staged:&Path, destination:&Path) -> Result<(),String> {
    for path in [current,staged] {
        let meta=fs::symlink_metadata(path).map_err(|e|e.to_string())?;
        if !path.is_absolute() || !meta.is_file() || meta.file_type().is_symlink() { return Err("Invalid executable update path".into()); }
    }
    if !destination.is_absolute() || destination.parent()!=current.parent() || destination==staged {
        return Err("Invalid executable update path".into());
    }
    if destination!=current && fs::symlink_metadata(destination).is_ok() {
        return Err("The new launcher filename already exists; move that file before updating".into());
    }
    let directory=staged.parent().ok_or("Invalid staging directory")?;
    let ready=directory.join("ready");
    let log=directory.join("apply.log");
    let backup=PathBuf::from(format!("{}.previous",current.display()));
    if let Ok(meta)=fs::symlink_metadata(&backup) {
        if !meta.is_file() || meta.file_type().is_symlink() { return Err("Invalid update backup path".into()); }
    }
    fs::write(directory.join("destination"),destination.as_os_str().to_str().ok_or("Invalid executable update path")?).map_err(|e|e.to_string())?;
    let mut command=helper(current,destination,staged,&backup,&ready)?;
    #[cfg(not(windows))]
    command.stdout(fs::File::create(&log).map_err(|e|e.to_string())?).stderr(fs::File::options().append(true).open(&log).map_err(|e|e.to_string())?);
    #[cfg(windows)]
    {
        // Do not retain open log handles across the Windows restart: they would
        // prevent the replacement process from removing its staging directory.
        fs::write(&log, b"").map_err(|e|e.to_string())?;
        command.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    }
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
fn helper(current:&Path,destination:&Path,staged:&Path,backup:&Path,ready:&Path)->Result<Command,String> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(staged,fs::Permissions::from_mode(0o755)).map_err(|e|e.to_string())?;
    let script=staged.parent().unwrap().join("apply.sh");
    fs::write(&script,include_str!("linux_update.sh")).map_err(|e|e.to_string())?;
    let mut c=Command::new("/bin/sh");
    c.arg(script).arg(std::process::id().to_string()).arg(destination).arg(staged).arg(backup).arg(ready).arg(current);
    super::clean_child_environment(&mut c);
    Ok(c)
}
#[cfg(target_os="windows")]
fn helper(current:&Path,destination:&Path,staged:&Path,backup:&Path,ready:&Path)->Result<Command,String> {
    use base64::Engine;
    use std::os::windows::process::CommandExt;
    let quote=|p:&Path| format!("'{}'",p.to_string_lossy().replace('\'',"''"));
    let script=format!("$oldPid={};$current={};$destination={};$staged={};$backup={};$ready={};\n{}",std::process::id(),quote(current),quote(destination),quote(staged),quote(backup),quote(ready),include_str!("windows/update.ps1"));
    let encoded=base64::engine::general_purpose::STANDARD.encode(script.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>());
    let mut c=Command::new("powershell.exe");
    c.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-EncodedCommand",&encoded]).creation_flags(0x08000000);
    Ok(c)
}
#[cfg(not(any(target_os="windows",target_os="linux")))]
fn helper(_: &Path,_:&Path,_:&Path,_:&Path,_:&Path)->Result<Command,String>{Err("Unsupported update platform".into())}

#[cfg(all(test,target_os="linux"))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    fn folder()->PathBuf{
        let p=std::env::temp_dir().join(format!("update test ' {}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&p).unwrap();p
    }
    #[test]
    fn replacement_uses_new_name_preserves_backup_working_directory_and_steam_environment(){
        let root=folder();let dest=root.join("custom ' name.AppImage");let staged=root.join("download.AppImage");let new_name=root.join("launcher-1.0.2.AppImage");let backup=root.join("custom ' name.AppImage.previous");
        fs::write(&dest,b"old binary").unwrap();fs::write(&backup,b"older binary").unwrap();
        fs::write(&staged,b"#!/bin/sh\nprintf '%s' \"$SteamGameId\" > result\n[ -z \"${APPIMAGE:-}\" ]\n").unwrap();
        fs::set_permissions(&staged,fs::Permissions::from_mode(0o755)).unwrap();
        let status=Command::new("/bin/sh").arg("-c").arg(include_str!("linux_update.sh")).arg("update").arg("2147483647").arg(&new_name).arg(&staged).arg(&backup).arg(root.join("ready")).arg(&dest).env("SteamGameId","123456").env("APPIMAGE","stale").status().unwrap();
        assert!(status.success());assert_eq!(fs::read(&backup).unwrap(),b"old binary");assert_eq!(fs::read(root.join("result")).unwrap(),b"123456");assert!(new_name.is_file());assert!(!dest.exists());assert!(!staged.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_move_restores_original(){
        let root=folder();let dest=root.join("launcher.AppImage");let backup=root.join("launcher.AppImage.previous");
        fs::write(&dest,b"original").unwrap();
        let output=Command::new("/bin/sh").arg("-c").arg(include_str!("linux_update.sh")).arg("update").arg("2147483647").arg(&dest).arg(root.join("missing")).arg(&backup).arg(root.join("ready")).arg(&dest).output().unwrap();
        assert!(!output.status.success());assert_eq!(fs::read(&dest).unwrap(),b"original");assert!(!backup.exists());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn rejects_links_before_spawning(){
        let root=folder();let dest=root.join("launcher.AppImage");let staged=root.join("new");
        fs::write(&dest,b"original").unwrap();std::os::unix::fs::symlink(&dest,&staged).unwrap();
        assert!(replace_after_exit(&dest,&staged,&root.join("updated.AppImage")).is_err());assert_eq!(fs::read(&dest).unwrap(),b"original");fs::remove_dir_all(root).unwrap();
    }
}

/// Called after the replacement application has initialized successfully.
/// Only the handoff directory for this exact executable can be removed.
pub fn cleanup_completed_update()->Result<(),String> {
    let Some(directory)=std::env::var_os("MCLAUNCHER_UPDATE_DIRECTORY") else {return Ok(());};
    std::env::remove_var("MCLAUNCHER_UPDATE_DIRECTORY");
    let (_,executable)=target()?;
    cleanup_directory(&executable,&PathBuf::from(directory))
}
fn cleanup_directory(executable:&Path,directory:&Path)->Result<(),String> {
    if directory.parent()!=executable.parent() || !directory.file_name().is_some_and(|n|n.to_string_lossy().starts_with(".launcher-update-")) {
        return Err("Invalid update cleanup directory".into());
    }
    let meta=fs::symlink_metadata(directory).map_err(|e|e.to_string())?;
    let marker=directory.join("destination");
    if !meta.is_dir() || meta.file_type().is_symlink() || fs::symlink_metadata(&marker).map_err(|e|e.to_string())?.file_type().is_symlink() || fs::read_to_string(marker).map_err(|e|e.to_string())? != executable.to_str().ok_or("Invalid executable update path")? {
        return Err("Invalid update cleanup directory".into());
    }
    fs::remove_dir_all(directory).map_err(|e|e.to_string())
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;
    #[test]
    fn completed_update_removes_all_staging_files_but_retains_executables() {
        let root=std::env::temp_dir().join(format!("update-cleanup-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let dir=root.join(".launcher-update-test");fs::create_dir_all(&dir).unwrap();
        let exe=root.join("launcher-1.0.1.exe");let backup=root.join("launcher-1.0.0.exe.previous");
        fs::write(&exe,b"new").unwrap();fs::write(&backup,b"old").unwrap();
        fs::write(dir.join("destination"),exe.to_str().unwrap()).unwrap();
        for name in ["apply.log","apply.sh","ready","download.partial"] {fs::write(dir.join(name),b"temporary").unwrap();}
        cleanup_directory(&exe,&dir).unwrap();
        assert!(!dir.exists());assert_eq!(fs::read(exe).unwrap(),b"new");assert_eq!(fs::read(backup).unwrap(),b"old");
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn cleanup_rejects_unowned_or_different_update_directory() {
        let root=std::env::temp_dir().join(format!("update-cleanup-reject-{}",std::process::id()));
        let dir=root.join(".launcher-update-test");fs::create_dir_all(&dir).unwrap();
        let exe=root.join("launcher.exe");fs::write(dir.join("destination"),"other executable").unwrap();
        assert!(cleanup_directory(&exe,&dir).is_err());assert!(dir.exists());
        assert!(cleanup_directory(&exe,&root).is_err());fs::remove_dir_all(root).unwrap();
    }
}
