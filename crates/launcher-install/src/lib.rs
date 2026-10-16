//! Installer core: no UI, authentication, shell scripts or game process launching.
pub mod self_update;
pub mod content;
pub mod content_source;
pub mod download;
pub mod java;
pub mod minecraft;
pub mod loaders;
pub mod forge;
use serde::{Deserialize, Serialize};
use std::{
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
pub type Result<T> = std::result::Result<T, String>;
pub fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err("Task cancelled".into())
    } else {
        Ok(())
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub phase: String,
    pub completed: u64,
    pub total: u64,
}
pub type Report<'a> = dyn Fn(Progress) + Send + Sync + 'a;
pub fn phase(report: &Report<'_>, message: impl Into<String>) {
    report(Progress {
        phase: message.into(),
        ..Progress::default()
    });
}
/// The root is trusted and may itself be a user symlink; remote paths may not traverse links.
pub fn safe_path(root: &Path, relative: &str) -> Result<PathBuf> {
    let path = Path::new(relative);
    if relative.is_empty()
        || relative.contains(['\\', '\0', ':'])
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("Unsafe file path: {relative}"));
    }
    let mut out = root.to_path_buf();
    for component in path.components() {
        out.push(component);
        match std::fs::symlink_metadata(&out) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(format!("Installation path contains a symbolic link: {}", out.display()))
            }
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => return Err(err(e)),
            _ => {}
        }
    }
    Ok(out)
}
pub fn atomic_json<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    use std::io::Write;
    let parent = path.parent().ok_or("Invalid path")?;
    std::fs::create_dir_all(parent).map_err(err)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(err)?;
    temp.write_all(&serde_json::to_vec_pretty(data).map_err(err)?)
        .map_err(err)?;
    temp.as_file().sync_all().map_err(err)?;
    temp.persist(path).map_err(err)?;
    Ok(())
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Platform {
    pub os: &'static str,
    pub architecture: &'static str,
    pub supported: bool,
}
pub fn platform() -> Platform {
    Platform {
        os: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        supported: launcher_platform::INSTALLATION_SUPPORTED,
    }
}
pub fn require_platform() -> Result<()> {
    if !platform().supported {
        return Err("Installation requires a supported Windows or Linux x86_64 build".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_cannot_escape_or_follow_links() {
        let dir = tempfile::tempdir().unwrap();
        for p in [
            "../outside",
            "/absolute",
            "a/../../outside",
            "C:/evil",
            "a\\b",
            "",
        ] {
            assert!(safe_path(dir.path(), p).is_err());
        }
        assert!(safe_path(dir.path(), "assets/objects/ab/cd").is_ok());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink("/tmp", dir.path().join("link")).unwrap();
            assert!(safe_path(dir.path(), "link/file").is_err());
        }
    }
}
