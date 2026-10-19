use std::{fs, path::{Path, PathBuf}};
use launcher_core::storage::atomic_write;
fn err(e:impl std::fmt::Display)->String {e.to_string()}
#[derive(Clone)]
pub struct Integration { pub script:PathBuf, pub desktop:PathBuf }
fn quote_shell(value: &str) -> String { format!("'{}'", value.replace('\'', "'\"'\"'")) }
fn image_text(image: &Path) -> Result<&str, String> {
    let text = image.to_str().ok_or("AppImage path is not valid UTF-8")?;
    if !image.is_absolute() || !image.is_file() || text.contains(['\n', '\r', '\0']) { return Err("Invalid AppImage path".into()); }
    Ok(text)
}
pub fn script(image: &Path) -> Result<String, String> {
    image_text(image)?;
    let filename = image.file_name().and_then(|s|s.to_str()).ok_or("Invalid launcher filename")?;
    Ok(format!("#!/bin/sh\nset -eu\nbase=$(CDPATH= cd -- \"$(dirname -- \"$0\")\" && pwd -P)\nimage=\"$base\"/{}\nexec \"$image\" \"$@\"\n", quote_shell(filename)))
}
impl Integration {
    /// Wayland resolves window icons through a desktop file matching the GTK ID.
    /// Keep the existing launcher entry (and Steam references) stable; the alias
    /// is hidden from application menus to avoid a duplicate launcher entry.
    pub fn register_identity(&self, desktop_id: &str) -> Result<(), String> {
        if desktop_id.is_empty() || !desktop_id.bytes().all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c)) {
            return Err("Invalid desktop application ID".into());
        }
        let alias = self.desktop.with_file_name(format!("{desktop_id}.desktop"));
        if alias == self.desktop { return Ok(()); }
        let entry = fs::read_to_string(&self.desktop).map_err(err)?;
        atomic_write(&alias, format!("{entry}NoDisplay=true\nStartupWMClass={desktop_id}\n").as_bytes())
    }
    pub fn register(&self, image: &Path) -> Result<(), String> {
        let content = script(image)?;
        atomic_write(&self.script, content.as_bytes())?;
        #[cfg(unix)] {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&self.script, fs::Permissions::from_mode(0o700)).map_err(err)?;
        }
        let path = self.script.to_str().ok_or("Launcher script path is not valid UTF-8")?;
        if path.contains(['\n', '\r', '\0']) { return Err("Invalid launcher script path".into()); }
        // Desktop Exec quoting is not shell quoting. Escape field codes separately.
        let escaped = path.replace('\\', "\\\\\\\\").replace('"', "\\\\\"").replace('`', "\\\\`").replace('$', "\\\\$").replace('%', "%%");
        let icon = self.script.with_file_name("icon.png");
        let icon_entry = if icon.is_file() {
            format!("Icon={}\n", icon.to_str().ok_or("Icon path is not valid UTF-8")?.replace('\\', "\\\\"))
        } else { String::new() };
        atomic_write(&self.desktop, format!("[Desktop Entry]\nType=Application\nName=MC Launcher Advanced\nComment=A third-party powered launcher for Minecraft Java Edition with an interface similar to the official.\nExec=\"{escaped}\"\n{icon_entry}Terminal=false\nCategories=Game;\n").as_bytes())
    }
}

#[cfg(test)] mod tests {
    use super::*;
    use launcher_core::storage::Storage;
    fn stamp()->String {format!("{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())}
    #[test] fn desktop_identity_keeps_icon_and_portable_entry() {
        let root=std::env::temp_dir().join(format!("desktop-identity-{}",stamp()));
        fs::create_dir_all(&root).unwrap();
        let image=root.join("launcher.AppImage"); fs::write(&image,b"package").unwrap();
        fs::write(root.join("icon.png"),b"icon").unwrap();
        let integration=Integration {script:root.join("launch.sh"),desktop:root.join("legacy.desktop")};
        integration.register(&image).unwrap();
        integration.register_identity("dev.personal.javalauncher").unwrap();
        let legacy=fs::read_to_string(&integration.desktop).unwrap();
        let alias=fs::read_to_string(root.join("dev.personal.javalauncher.desktop")).unwrap();
        assert!(alias.starts_with(&legacy));
        assert!(alias.contains("Icon="));
        assert!(alias.contains("NoDisplay=true\nStartupWMClass=dev.personal.javalauncher\n"));
        assert!(!legacy.contains("NoDisplay"));
        assert!(integration.register_identity("../bad").is_err());
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test] fn moved_script_preserves_steam_environment_and_arguments() {
        use std::os::unix::fs::PermissionsExt;
        let base=std::env::temp_dir().join(format!("portable script {}",stamp()));
        fs::create_dir_all(&base).unwrap();
        let image=base.join("launcher's test.AppImage");
        fs::write(&image,b"#!/bin/sh\nprintf '%s\\n' \"$SteamGameId\" \"$1\"\n").unwrap();
        fs::set_permissions(&image,fs::Permissions::from_mode(0o700)).unwrap();
        let integration=Integration {script:base.join("launch.sh"),desktop:base.join("test.desktop")};
        integration.register(&image).unwrap();
        let moved=base.with_extension("moved"); fs::rename(&base,&moved).unwrap();
        let output=std::process::Command::new(moved.join("launch.sh")).current_dir("/").env("SteamGameId","123456").arg("argument with spaces").output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(),"123456\nargument with spaces\n");
        fs::remove_dir_all(moved).unwrap();
    }
    #[test] fn portable_storage_preserves_image_and_updates_in_place() {
        let root = std::env::temp_dir().join(format!("portable-{}",stamp()));
        fs::create_dir_all(&root).unwrap();
        let image=root.join("launcher.AppImage"); fs::write(&image,b"package").unwrap();
        let integration=Integration {script:root.join("launch.sh"),desktop:root.join("app.desktop")};
        integration.register(&image).unwrap();
        let store=Storage::open(&root).unwrap();
        assert_eq!(store.location.root,root.join(".minecraft"));
        assert_eq!(fs::read(&image).unwrap(),b"package");
        assert!(!root.join(".mclauncher-advanced").exists());
        let script=fs::read_to_string(integration.script).unwrap();
        assert!(script.contains("exec \"$image\" \"$@\""));
        assert!(!script.contains(root.to_str().unwrap()));
        fs::remove_dir_all(root).unwrap();
    }
}

fn available_bytes(contents: &str) -> Result<u64, String> {
    contents.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        if fields.next()? != "MemAvailable:" { return None; }
        let kib = fields.next()?.parse::<u64>().ok()?;
        if fields.next()? != "kB" { return None; }
        kib.checked_mul(1024)
    }).ok_or_else(|| "Available memory could not be detected".into())
}

pub fn available_memory_bytes() -> Result<u64, String> {
    #[cfg(target_os = "linux")]
    { available_bytes(&std::fs::read_to_string("/proc/meminfo").map_err(|e| e.to_string())?) }
    #[cfg(not(target_os = "linux"))]
    { Err("Memory detection is not supported on this platform yet".into()) }
}

#[cfg(test)]
mod memory_tests {
    use super::*;
    #[test]
    fn uses_available_memory_not_total_or_free() {
        assert_eq!(available_bytes("MemTotal: 10000 kB\nMemFree: 100 kB\nMemAvailable: 2048 kB\n").unwrap(), 2097152);
        assert!(available_bytes("MemTotal: 10000 kB").is_err());
        assert!(available_bytes("MemAvailable: bad kB").is_err());
    }
}

pub fn steam_keyboard(show:bool)->Result<(),String> {
        // Fixed executable/arguments; never pass field contents or renderer-supplied URLs.
        let mut child = std::process::Command::new("/usr/bin/steam")
            .args(["-ifrunning", if show { "steam://open/keyboard?XPosition=0&YPosition=0&Width=0&Height=0&Mode=0" } else { "steam://close/keyboard" }])
            .env_remove("LD_PRELOAD").env_remove("LD_LIBRARY_PATH")
            .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
            .spawn().map_err(|e| format!("Steam keyboard is unavailable; use a physical keyboard: {e}"))?;
        let started = std::time::Instant::now();
        loop {
            if let Some(status) = child.try_wait().map_err(|e| e.to_string())? { return if status.success() { Ok(()) } else { Err("Steam rejected the keyboard request; use a physical keyboard".into()) }; }
            if started.elapsed() > std::time::Duration::from_millis(2500) { let _ = child.kill(); let _ = child.wait(); return Err("Steam keyboard request timed out; use a physical keyboard".into()); }
            std::thread::sleep(std::time::Duration::from_millis(25));
        }
}

// renameat2 keeps both files intact even if an external manager creates the target
// between our scan and the toggle.
#[cfg(target_os="linux")]
pub fn move_without_replace(source: &Path, destination: &Path) -> Result<(),String> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let source=CString::new(source.as_os_str().as_bytes()).map_err(err)?;
    let destination=CString::new(destination.as_os_str().as_bytes()).map_err(err)?;
    let result=unsafe {libc::renameat2(libc::AT_FDCWD,source.as_ptr(),libc::AT_FDCWD,destination.as_ptr(),libc::RENAME_NOREPLACE)};
    if result==0 {return Ok(())}
    let error=std::io::Error::last_os_error();
    if error.kind()==std::io::ErrorKind::AlreadyExists {return Err("A file with this name already exists in the destination; no files were replaced".into());}
    Err(err(error))
}
