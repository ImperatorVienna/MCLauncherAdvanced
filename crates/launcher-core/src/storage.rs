//! Platform-independent portable launcher storage.
use crate::Store;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::{Path, PathBuf}};
pub const LIBRARY: &str = "launcher-library.json";
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location { pub root: PathBuf, pub warning: Option<String> }
fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn stamp() -> String { format!("{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()) }
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    fs::create_dir_all(path.parent().ok_or("Invalid file path")?).map_err(err)?;
    let temp = path.with_extension(format!("{}.tmp", stamp()));
    let result = (|| {
        let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&temp).map_err(err)?;
        file.write_all(bytes).map_err(err)?;
        file.sync_all().map_err(err)?;
        fs::rename(&temp, path).map_err(err)
    })();
    if result.is_err() { let _ = fs::remove_file(temp); }
    result
}
pub struct Storage { pub location: Location, pub launcher_root: PathBuf }
impl Storage {
    pub fn open(base: &Path) -> Result<Self, String> {
        let launcher_root = base.canonicalize().map_err(err)?;
        let root = launcher_root.join(".minecraft");
        fs::create_dir_all(&root).map_err(err)?;
        let root = root.canonicalize().map_err(err)?;
        if root == launcher_root || root.parent().is_none() { return Err("Invalid .minecraft directory".into()); }
        Store::new(launcher_root.join(LIBRARY)).load()?;
        Ok(Self { location: Location { root, warning: None }, launcher_root })
    }
    pub fn store(&self) -> Store { Store::new(self.launcher_root.join(LIBRARY)) }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn managed_java_resolves_after_directory_move() {
        let base=std::env::temp_dir().join(format!("portable-java-{}",stamp()));
        fs::create_dir_all(&base).unwrap();
        let storage=Storage::open(&base).unwrap();
        let item=crate::Instance {id:"one".into(),name:"one".into(),directory_name:"one".into(),minecraft_version:"1.21.1".into(),icon:"grass".into(),loader:None,java_path:Some(base.join("runtimes/java/jdk/bin/java").to_string_lossy().into()),launch_settings:Default::default()};
        storage.store().save(crate::Library {instances:vec![item],selected_id:Some("one".into()),..Default::default()}).unwrap();
        let raw=fs::read_to_string(base.join(LIBRARY)).unwrap();
        assert!(raw.contains("runtimes/java/jdk/bin/java")); assert!(!raw.contains(base.to_str().unwrap()));
        let moved=base.with_extension("moved"); fs::rename(&base,&moved).unwrap();
        let loaded=Storage::open(&moved).unwrap().store().load().unwrap();
        assert_eq!(loaded.instances[0].java_path.as_deref(),moved.join("runtimes/java/jdk/bin/java").to_str());
        fs::remove_dir_all(moved).unwrap();
    }
}
