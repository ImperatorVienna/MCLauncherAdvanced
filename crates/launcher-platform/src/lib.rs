//! Host operating-system adapters. No game, account or UI policy belongs here.
#[cfg(any(target_os="linux",windows))]
pub mod gamepad;
use std::path::{Path,PathBuf};
#[cfg(target_os="linux")]
pub mod linux;
#[cfg(any(windows,test))]
pub mod windows;

pub fn launcher_directory()->Result<PathBuf,std::io::Error> {
    let executable=packaged_executable()?.unwrap_or(std::env::current_exe()?);
    executable.parent().map(Path::to_path_buf).ok_or_else(||std::io::Error::other("Invalid launcher location"))
}
fn packaged_executable()->Result<Option<PathBuf>,std::io::Error> {
    #[cfg(all(target_os="linux",not(debug_assertions)))]
    {return std::env::var_os("APPIMAGE").map(PathBuf::from).map(|p|p.canonicalize()).transpose();}
    #[cfg(not(all(target_os="linux",not(debug_assertions))))]
    {Ok(None)}
}
pub fn register(root:&Path,data:&Path,icon:&[u8],windows_icon:&[u8],desktop_id:&str)->Result<(),String> {
    #[cfg(target_os="linux")]
    if let Some(image)=packaged_executable().map_err(|e|e.to_string())? {
        launcher_core::storage::atomic_write(&root.join("icon.png"),icon)?;
        let _=windows_icon;
        let integration=linux::Integration{script:root.join("launch.sh"),desktop:data.join("applications/minecraft-java-launcher.desktop")};
        integration.register(&image)?;
        integration.register_identity(desktop_id)?;
    }
    #[cfg(windows)]
    { let _=(data,icon,desktop_id); if !cfg!(debug_assertions) {
        launcher_core::storage::atomic_write(&root.join("icon.ico"),windows_icon)?;
        windows::register(root)?;
    } }
    #[cfg(not(any(target_os="linux",windows)))]
    let _=(root,data,icon,windows_icon,desktop_id);
    Ok(())
}
pub fn open_external(target:impl AsRef<std::ffi::OsStr>)->Result<(),String> {
    #[cfg(target_os="linux")]
    {
        let status=std::process::Command::new("xdg-open").arg(target)
            .env_remove("LD_LIBRARY_PATH").env_remove("LD_PRELOAD")
            .stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .status().map_err(|e|e.to_string())?;
        if status.success(){Ok(())}else{Err("Could not open the file; check the system default application".into())}
    }
    #[cfg(windows)]
    {windows::open_external(target)}
    #[cfg(not(any(target_os="linux",windows)))]
    {let _=target;Err("Opening external resources is not supported on this platform yet".into())}
}
pub fn kernel_version()->Result<String,String> {
    #[cfg(target_os="linux")]
    {std::fs::read_to_string("/proc/sys/kernel/osrelease").map_err(|e|e.to_string())}
    #[cfg(windows)]
    {windows::kernel_version()}
    #[cfg(not(any(target_os="linux",windows)))]
    {Err("OS version detection is not supported on this platform yet".into())}
}
#[cfg(target_os="linux")]
pub use linux::{available_memory_bytes,steam_keyboard,move_without_replace};
#[cfg(windows)]
pub use windows::{available_memory_bytes,steam_keyboard,move_without_replace,spawn_installer,kill_installer};
#[cfg(not(any(target_os="linux",windows)))]
pub fn available_memory_bytes()->Result<u64,String>{Err("Memory detection is not supported on this platform yet".into())}
#[cfg(not(any(target_os="linux",windows)))]
pub fn steam_keyboard(_:bool)->Result<(),String>{Err("Steam keyboard is not supported on this platform yet".into())}
#[cfg(not(any(target_os="linux",windows)))]
pub fn move_without_replace(_: &Path,_: &Path)->Result<(),String>{Err("Content management is not supported on this platform".into())}

pub fn protect_private_file(file:&std::fs::File)->Result<(),String> {
    #[cfg(unix)]
    {use std::os::unix::fs::PermissionsExt;file.set_permissions(std::fs::Permissions::from_mode(0o600)).map_err(|e|e.to_string())}
    #[cfg(windows)]
    {windows::protect_private_file(file)}
    #[cfg(not(any(unix,windows)))]
    {let _=file;Err("Private account storage is not supported on this platform yet".into())}
}
pub fn prepare_process_group(command:&mut std::process::Command)->Result<(),String> {
    #[cfg(target_os="linux")]
    {use std::os::unix::process::CommandExt;command.process_group(0);Ok(())}
    #[cfg(not(target_os="linux"))]
    {let _=command;Err("Installer process isolation is not supported on this platform yet".into())}
}
pub fn kill_process_group(child:&mut std::process::Child) {
    #[cfg(target_os="linux")]
    unsafe {libc::kill(-(child.id() as i32),libc::SIGKILL);}
    let _=child.kill();let _=child.wait();
}
pub fn clean_child_environment(command:&mut std::process::Command)->&mut std::process::Command {
    #[cfg(target_os="linux")]
    command.env_remove("LD_LIBRARY_PATH").env_remove("LD_PRELOAD");
    #[cfg(windows)]
    {use std::os::windows::process::CommandExt;command.creation_flags(0x08000000);}
    command
}
pub fn java_search_roots()->Vec<PathBuf> {
    #[cfg(target_os="linux")]
    {vec!["/usr/lib/jvm","/usr/lib64/jvm","/usr/java","/opt/java","/opt/jdk"].into_iter().map(PathBuf::from).collect()}
    #[cfg(windows)]
    {std::env::var_os("ProgramFiles").map(PathBuf::from).into_iter().flat_map(|root|["Java","Eclipse Adoptium","Microsoft","BellSoft","Amazon Corretto"].map(|name|root.join(name))).collect()}
    #[cfg(not(any(target_os="linux",windows)))]
    {vec![]}
}
pub const JAVA_EXECUTABLE:&str=if cfg!(windows){"java.exe"}else{"java"};
pub const INSTALLATION_SUPPORTED:bool=cfg!(all(target_arch="x86_64",any(all(target_os="linux",target_env="gnu"),target_os="windows")));

pub fn set_extracted_permissions(path:&Path,mode:u32)->Result<(),String> {
    #[cfg(unix)]
    {use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(path,std::fs::Permissions::from_mode(mode & 0o777)).map_err(|e|e.to_string())}
    #[cfg(not(unix))]
    {let _=(path,mode);Ok(())}
}
pub fn create_runtime_symlink(target:&Path,path:&Path)->Result<(),String> {
    #[cfg(unix)]
    {std::os::unix::fs::symlink(target,path).map_err(|e|e.to_string())}
    #[cfg(not(unix))]
    {let _=(target,path);Err("This build does not support the Java extraction strategy".into())}
}

#[cfg(not(windows))]
pub fn spawn_installer(command:&mut std::process::Command)->Result<std::process::Child,String> {
    prepare_process_group(command)?;command.spawn().map_err(|e|e.to_string())
}
#[cfg(not(windows))]
pub fn kill_installer(child:&mut std::process::Child){kill_process_group(child)}
pub const OS_NAME:&str=if cfg!(windows){"windows"}else{std::env::consts::OS};
pub const CLASSPATH_SEPARATOR:&str=if cfg!(windows){";"}else{":"};
pub const JAVA_ARCHIVE_EXTENSION:&str=if cfg!(windows){"zip"}else{"tar.gz"};

/// JVM arguments use conventional paths, not Rust's verbatim canonical prefix.
pub fn child_path(path:&Path)->String {
    let text=path.to_string_lossy();
    #[cfg(windows)]
    {if let Some(rest)=text.strip_prefix(r"\\?\UNC\"){return format!(r"\\{}",rest)}if let Some(rest)=text.strip_prefix(r"\\?\"){return rest.into()}}
    text.into_owned()
}

pub mod updater;
