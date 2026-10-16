//! Windows portable entry generation. No navigation or game workflow policy.
#[cfg(windows)]
use std::path::Path;
use base64::Engine;
fn encoded(script:&str)->String {
    base64::engine::general_purpose::STANDARD.encode(script.encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<_>>())
}
pub fn launch_script()->String {
    format!("@echo off\r\nsetlocal DisableDelayedExpansion\r\nset \"MCLAUNCHER_ROOT=%~dp0\"\r\n\"%SystemRoot%\\System32\\WindowsPowerShell\\v1.0\\powershell.exe\" -NoLogo -NoProfile -NonInteractive -EncodedCommand {}\r\nif errorlevel 1 (\r\n  pause\r\n  exit /b 1\r\n)\r\n",encoded(include_str!("windows/launch.ps1")))
}
#[cfg(windows)]
pub fn register(root:&Path)->Result<(),String> {
    let executable=std::env::current_exe().map_err(|e|e.to_string())?;
    let name=executable.file_name().and_then(|n|n.to_str()).ok_or("Invalid launcher filename")?;
    launcher_core::storage::atomic_write(&root.join("launcher-executable.txt"),name.as_bytes())?;
    launcher_core::storage::atomic_write(&root.join("launch.bat"),launch_script().as_bytes())?;
    let system=std::env::var_os("SystemRoot").ok_or("Windows system directory is unavailable")?;
    let status=std::process::Command::new(Path::new(&system).join("System32/WindowsPowerShell/v1.0/powershell.exe"))
        .args(["-NoLogo","-NoProfile","-NonInteractive","-EncodedCommand",&encoded(include_str!("windows/register.ps1"))])
        .env("MCLAUNCHER_ROOT",root).status().map_err(|e|e.to_string())?;
    if status.success(){Ok(())}else{Err("Could not create the launcher desktop shortcut".into())}
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bat_contains_only_fixed_commands_and_encoded_payload() {
        let bat=launch_script();
        assert!(bat.contains("setlocal DisableDelayedExpansion"));
        assert!(bat.contains("MCLAUNCHER_ROOT=%~dp0"));
        assert!(include_str!("windows/launch.ps1").contains("$process.WaitForExit()"));
        assert!(include_str!("windows/launch.ps1").contains("$start.UseShellExecute = $false"));
        let command=bat.split("-EncodedCommand ").nth(1).unwrap().lines().next().unwrap();
        let bytes=base64::engine::general_purpose::STANDARD.decode(command).unwrap();
        let units=bytes.chunks_exact(2).map(|c|u16::from_le_bytes([c[0],c[1]])).collect::<Vec<_>>();
        assert_eq!(String::from_utf16(&units).unwrap(),include_str!("windows/launch.ps1"));
    }
}

#[cfg(windows)]
mod runtime;
#[cfg(windows)]
pub use runtime::*;
