//! Local configuration only. No Tauri, network, authentication or game launch dependencies.
pub mod launch_settings;
pub const INSTANCE_ICONS: &[&str] = &["grass", "sand", "cobblestone", "oak", "brick", "diamond", "emerald", "redstone", "obsidian", "diamond_sword", "iron_pickaxe", "bow", "apple", "golden_apple", "ender_pearl", "enchanted_book", "coal", "iron_ingot", "gold_ingot", "lapis_lazuli", "totem_of_undying", "lava_bucket", "fishing_rod", "blaze_rod", "nether_star", "anvil", "crafting_table", "furnace", "wooden_axe", "saddle"];
use launch_settings::LaunchSettings;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModLoader { pub kind: String, pub version: String }
impl ModLoader {
    pub fn validate(&self) -> Result<(), String> {
        if !["fabric", "quilt", "forge", "neoforge"].contains(&self.kind.as_str()) || self.version.is_empty() || self.version.len() > 80 || !self.version.bytes().all(|b| b.is_ascii_alphanumeric() || b".-+".contains(&b)) {
            return Err("Invalid mod loader selection".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub icon: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loader: Option<ModLoader>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java_path: Option<String>,
    pub directory_name: String,
    #[serde(default)]
    pub launch_settings: LaunchSettings,
}
impl Instance {
    pub fn directory(&self) -> &str { &self.directory_name }
}
pub fn validate_directory_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name == "." || name == ".." || name.trim() != name || name.ends_with('.') || name.len() > 240 || name.chars().any(|c| c.is_control() || "/\\:*?\"<>|".contains(c)) {
        return Err("Instance names cannot contain path separators or special filename characters".into());
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Library {
    pub schema_version: u32,
    pub revision: u64,
    pub instances: Vec<Instance>,
    pub selected_id: Option<String>,
    pub last_launched_id: Option<String>,
}
impl Default for Library {
    fn default() -> Self {
        Self { schema_version: 2, revision: 0, instances: vec![], selected_id: None, last_launched_id: None }
    }
}
impl Library {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 2 { return Err("Unsupported instance library schema version".into()); }
        if self.instances.len() > 1000 { return Err("The instance limit is 1000".into()); }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        for item in &self.instances {
            if item.id.is_empty() || item.id.len() > 80 || !item.id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-') || !ids.insert(item.id.as_str()) {
                return Err("Invalid or duplicate instance ID".into());
            }
            if item.name.trim() != item.name || item.name.is_empty() || item.name.chars().count() > 64 || item.name.chars().any(char::is_control) || !names.insert(item.name.to_lowercase()) {
                return Err("Instance names must be nonempty, unique, and at most 64 characters".into());
            }
            if item.minecraft_version.is_empty() || item.minecraft_version.len() > 40 || !item.minecraft_version.bytes().all(|c| c.is_ascii_alphanumeric() || b" .-_".contains(&c)) {
                return Err("Invalid Minecraft version format".into());
            }
            validate_directory_name(&item.directory_name)?;
            item.launch_settings.validate()?;
            if let Some(loader) = &item.loader { loader.validate()?; }
            if item.java_path.as_ref().is_some_and(|p| !std::path::Path::new(p).is_absolute() || p.len() > 4096 || p.contains('\0')) { return Err("Invalid Java path".into()); }
            if !INSTANCE_ICONS.contains(&item.icon.as_str()) { return Err("Unknown instance icon".into()); }
        }
        for id in [&self.selected_id, &self.last_launched_id].into_iter().flatten() {
            if !ids.contains(id.as_str()) { return Err("Referenced instance does not exist".into()); }
        }
        if !self.instances.is_empty() && self.selected_id.is_none() { return Err("Select an instance".into()); }
        Ok(())
    }
}
/// Commands serialize access using a Tauri-managed mutex. Save validates before touching disk.
pub struct Store { path: PathBuf }
impl Store {
    pub fn new(path: PathBuf) -> Self { Self { path } }
    pub fn load(&self) -> Result<Library, String> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Library::default()),
            Err(e) => return Err(format!("Failed to read the instance library: {e}")),
        };
        let mut library: Library = serde_json::from_slice(&bytes).map_err(|e| format!("The instance library is corrupt; the original file was preserved: {e}"))?;
        for item in &mut library.instances {
            if let Some(path) = &item.java_path {
                let relative = std::path::Path::new(path);
                if !relative.is_absolute() {
                    if !relative.starts_with("runtimes/java") || relative.components().any(|c| !matches!(c, std::path::Component::Normal(_))) { return Err("Invalid Java path".into()); }
                    item.java_path = Some(self.path.parent().ok_or("Invalid library path")?.join(relative).to_string_lossy().into_owned());
                }
            }
        }
        library.validate()?;
        Ok(library)
    }
    pub fn save(&self, mut next: Library) -> Result<Library, String> {
        next.validate()?;
        let parent = self.path.parent().ok_or("Invalid instance library path")?;
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create the instance library directory: {e}"))?;
        // An OS lock also serializes separate launcher processes; dropping releases it.
        let lock = fs::OpenOptions::new().create(true).truncate(false).write(true)
            .open(self.path.with_extension("lock")).map_err(|e| format!("Failed to open the instance library lock: {e}"))?;
        lock.try_lock().map_err(|e| format!("Another process is saving the instance library; retry later: {e}"))?;
        let current = self.load()?;
        if next.revision != current.revision { return Err("The instance library changed; reload and retry".into()); }
        // Only a future successful-launch operation may write a new launch record.
        let retained_launch = current.last_launched_id.filter(|id| next.instances.iter().any(|i| &i.id == id));
        if next.last_launched_id != retained_launch { return Err("Editing instances cannot change launch history".into()); }
        next.revision = current.revision.checked_add(1).ok_or("Instance library revision overflow")?;
        let temp = self.path.with_extension("json.tmp");
        let result = (|| -> Result<(), String> {
            let mut file = fs::File::create(&temp).map_err(|e| e.to_string())?;
            let mut portable = next.clone();
            for item in &mut portable.instances {
                if let Some(path) = &item.java_path {
                    if let Ok(relative) = std::path::Path::new(path).strip_prefix(parent) {
                        if relative.starts_with("runtimes/java") { item.java_path = Some(relative.iter().map(|part|part.to_string_lossy()).collect::<Vec<_>>().join("/")); }
                    }
                }
            }
            file.write_all(&serde_json::to_vec_pretty(&portable).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            fs::rename(&temp, &self.path).map_err(|e| e.to_string())?;
            Ok(())
        })();
        if let Err(e) = result { let _ = fs::remove_file(temp); return Err(format!("Failed to save the instance library: {e}")); }
        Ok(next)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn store() -> Store {
        Store::new(std::env::temp_dir().join(format!("launcher-core-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())).join("library.json"))
    }
    fn example() -> Library {
        Library { instances: vec![Instance { id: "one".into(), name: "\u{539f}\u{7248}".into(), minecraft_version: "1.21.1".into(), icon: "grass".into(), java_path: None, loader: None, launch_settings: Default::default(), directory_name: "1.21.1".into() }], selected_id: Some("one".into()), ..Library::default() }
    }
    #[test]
    fn java_selection_is_optional_and_persisted() {
        let old = r#"{"id":"one","name":"\u6d4b\u8bd5","minecraftVersion":"1.21","directoryName":"test","icon":"grass"}"#;
        let mut item: Instance = serde_json::from_str(old).unwrap();
        assert!(item.java_path.is_none());
        item.java_path = Some("/opt/custom-jdk/bin/java".into());
        item.icon = "diamond".into();
        let restored: Instance = serde_json::from_slice(&serde_json::to_vec(&item).unwrap()).unwrap();
        assert_eq!(restored.java_path, item.java_path);
        let lib = Library { instances: vec![restored], selected_id: Some("one".into()), ..Library::default() };
        lib.validate().unwrap();
    }
    #[test] fn round_trip_conflict_and_delete() {
        let store = store(); assert!(store.load().unwrap().instances.is_empty());
        let saved = store.save(example()).unwrap(); assert_eq!(saved.revision, 1);
        assert_eq!(store.load().unwrap().instances[0].name, "\u{539f}\u{7248}");
        assert!(store.save(example()).is_err());
        assert!(store.save(Library { revision: 1, ..Library::default() }).unwrap().instances.is_empty());
        fs::remove_dir_all(store.path.parent().unwrap()).unwrap();
    }
    #[test] fn invalid_data_and_forged_launch_do_not_write() {
        let store = store(); let mut data = example(); data.last_launched_id = Some("one".into());
        assert!(store.save(data).is_err());
        let mut data = example(); data.instances.push(data.instances[0].clone()); assert!(store.save(data).is_err());
        assert!(!store.path.exists());
    }
    #[test] fn corrupt_file_is_preserved() {
        let store = store(); fs::create_dir_all(store.path.parent().unwrap()).unwrap(); fs::write(&store.path, b"broken").unwrap();
        assert!(store.load().is_err()); assert!(store.save(example()).is_err()); assert_eq!(fs::read(&store.path).unwrap(), b"broken");
        fs::remove_dir_all(store.path.parent().unwrap()).unwrap();
    }
}

pub mod storage;
