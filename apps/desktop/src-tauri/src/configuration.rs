//! Configuration lifecycle. Callers hold the storage mutex and exclude file tasks.
use launcher_core::{Instance, Library, Store};
use launcher_install::minecraft::instance_directory;
use std::{fs, path::Path};

pub fn create(store: &Store, root: &Path, mut item: Instance, revision: u64) -> Result<Library, String> {
    launcher_core::validate_directory_name(&item.name)?;
    if item.launch_settings.max_memory_mib.is_some() {
        item.launch_settings.validate_available_memory(crate::memory::available_memory_bytes()?)?;
    }
    let mut library = store.load()?;
    if library.revision != revision { return Err("The instance library changed; reload and retry".into()); }
    item.directory_name = item.name.clone();
    let directory = instance_directory(root, item.directory())?;
    // Check even untracked directories; never adopt or overwrite another launcher's data.
    let versions = launcher_install::safe_path(root, "versions")?;
    fs::create_dir_all(&versions).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(&versions).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_name().to_string_lossy().to_lowercase() == item.name.to_lowercase() {
            return Err(format!("Instance name conflict: \"{}\" already exists in versions; choose another name", item.name));
        }
    }
    if library.instances.iter().any(|i| i.name.to_lowercase() == item.name.to_lowercase() || i.directory().to_lowercase() == item.name.to_lowercase()) {
        return Err("Instance name conflict; choose another name".into());
    }
    library.selected_id = Some(item.id.clone());
    library.instances.push(item);
    library.validate()?;
    fs::create_dir(&directory).map_err(|e| format!("Failed to create the instance directory (possible name conflict): {e}"))?;
    match store.save(library) {
        Ok(saved) => Ok(saved),
        Err(e) => { let _ = fs::remove_dir(&directory); Err(e) }
    }
}

pub fn save_metadata(store: &Store, next: Library) -> Result<Library, String> {
    let current = store.load()?;
    if current.instances.len() != next.instances.len() { return Err("Use create or delete to manage instance directories".into()); }
    for item in &next.instances {
        let old = current.instances.iter().find(|i| i.id == item.id).ok_or("Use the create operation to add an instance")?;
        if old.launch_settings != item.launch_settings && item.launch_settings.max_memory_mib.is_some() {
            item.launch_settings.validate_available_memory(crate::memory::available_memory_bytes()?)?;
        }
        if old.loader != item.loader { return Err("Use a dedicated operation to change the mod loader".into()); }
        if old.minecraft_version != item.minecraft_version || old.directory_name != item.directory_name {
            return Err("The Minecraft version and directory name are immutable; create another instance".into());
        }
    }
    store.save(next)
}

pub fn delete(store: &Store, root: &Path, id: &str, revision: u64) -> Result<Library, String> {
    let mut library = store.load()?;
    if library.revision != revision { return Err("The instance library changed; reload and retry".into()); }
    let item = library.instances.iter().find(|i| i.id == id).ok_or("Instance not found")?;
    if library.instances.iter().any(|i| i.id != id && i.directory().eq_ignore_ascii_case(item.directory())) {
        return Err("Multiple instances share a directory; resolve the shared directory before deleting".into());
    }
    let directory = instance_directory(root, item.directory())?;
    // Keep the record if cleanup fails (including partial deletion), so retry remains possible.
    // remove_dir_all unlinks nested symlinks rather than following them; the root is checked above.
    if directory.exists() { fs::remove_dir_all(&directory).map_err(|e| format!("Failed to delete the instance directory; the instance was retained for retry: {e}"))?; }
    library.instances.retain(|i| i.id != id);
    if library.selected_id.as_deref() == Some(id) { library.selected_id = library.instances.first().map(|i| i.id.clone()); }
    if library.last_launched_id.as_deref() == Some(id) { library.last_launched_id = None; }
    store.save(library).map_err(|e| format!("The instance directory was deleted but its record could not be saved; retry deletion: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (std::path::PathBuf, Store) {
        let root = std::env::temp_dir().join(format!("configuration-lifecycle-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&root).unwrap();
        let store = Store::new(root.join("launcher-library.json"));
        (root, store)
    }
    fn item(id: &str, name: &str) -> Instance {
        Instance { id: id.into(), name: name.into(), minecraft_version: "26.3".into(), icon: "grass".into(), java_path: None, loader: None, launch_settings: Default::default(), directory_name: "1.21.1".into() }
    }
    #[test]
    fn named_directories_are_unique_and_deletion_preserves_shared_files() {
        let (root, store) = setup();
        let first = create(&store, &root, item("one", "\u{539f}\u{7248} \u{751f}\u{5b58}"), 0).unwrap();
        assert_eq!(first.instances[0].directory(), "\u{539f}\u{7248} \u{751f}\u{5b58}");
        assert!(root.join("versions/\u{539f}\u{7248} \u{751f}\u{5b58}").is_dir());
        assert!(create(&store, &root, item("two", "\u{539f}\u{7248} \u{751f}\u{5b58}"), 1).unwrap_err().contains("Instance name conflict"));
        fs::create_dir_all(root.join("versions/HMCL/saves")).unwrap();
        assert!(create(&store, &root, item("two", "hmcl"), 1).unwrap_err().contains("Instance name conflict"));
        let second = create(&store, &root, item("two", "\u{521b}\u{9020}"), 1).unwrap();
        for path in ["versions/\u{539f}\u{7248} \u{751f}\u{5b58}/saves/world", "versions/\u{539f}\u{7248} \u{751f}\u{5b58}/mods/test", "versions/\u{521b}\u{9020}/options.txt", "libraries/keep", "assets/keep"] {
            let path = root.join(path); fs::create_dir_all(path.parent().unwrap()).unwrap(); fs::write(path, b"keep").unwrap();
        }
        assert!(delete(&store, &root, "one", 1).is_err());
        assert!(root.join("versions/\u{539f}\u{7248} \u{751f}\u{5b58}/saves/world").exists());
        let next = delete(&store, &root, "one", second.revision).unwrap();
        assert_eq!(next.instances.len(), 1);
        assert!(!root.join("versions/\u{539f}\u{7248} \u{751f}\u{5b58}").exists());
        for path in ["versions/\u{521b}\u{9020}/options.txt", "versions/HMCL/saves", "libraries/keep", "assets/keep"] { assert!(root.join(path).exists()); }
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn invalid_names_and_metadata_bypass_cannot_touch_files() {
        let (root, store) = setup();
        for name in ["../escape", ".", "..", "a/b", "a\\b", "bad:", "bad."] { assert!(create(&store, &root, item("one", name), 0).is_err()); }
        assert!(store.load().unwrap().instances.is_empty());
        let saved = create(&store, &root, item("one", "test"), 0).unwrap();
        let mut next = saved.clone(); next.instances.clear(); next.selected_id = None;
        assert!(save_metadata(&store, next).is_err());
        let mut next = saved.clone(); next.instances[0].minecraft_version = "1.21".into();
        assert!(save_metadata(&store, next).is_err());
        let mut next = saved; next.instances[0].icon = "sand".into();
        assert!(save_metadata(&store, next).is_ok());
        assert!(root.join("versions/test").is_dir());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn launch_settings_persist_and_reject_invalid_updates() {
        let (root, store) = setup();
        let mut library = create(&store, &root, item("settings", "settings"), 0).unwrap();
        library.instances[0].launch_settings.display_mode = launcher_core::launch_settings::DisplayMode::Windowed;
        library.instances[0].launch_settings.width = 1920;
        library.instances[0].launch_settings.height = 1080;
        library.instances[0].launch_settings.remember_maximized = true;
        let saved = save_metadata(&store, library).unwrap();
        assert_eq!(store.load().unwrap().instances[0].launch_settings, saved.instances[0].launch_settings);
        let mut invalid = saved.clone();
        invalid.instances[0].launch_settings.width = 0;
        assert!(save_metadata(&store, invalid).is_err());
        let mut invalid = saved.clone();
        invalid.instances[0].launch_settings.max_memory_mib = Some(u32::MAX);
        assert!(save_metadata(&store, invalid).is_err());
        assert_eq!(store.load().unwrap().revision, saved.revision);
        fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn symlinks_and_shared_directories_cannot_delete_other_data() {
        let (root, store) = setup();
        let mut library = create(&store, &root, item("one", "test"), 0).unwrap();
        fs::create_dir(root.join("outside")).unwrap(); fs::write(root.join("outside/keep"), b"safe").unwrap();
        std::os::unix::fs::symlink(root.join("outside"), root.join("versions/test/link")).unwrap();
        library = delete(&store, &root, "one", library.revision).unwrap();
        assert!(root.join("outside/keep").exists());
        std::os::unix::fs::symlink(root.join("outside"), root.join("versions/test")).unwrap();
        assert!(create(&store, &root, item("one", "test"), library.revision).is_err());
        let mut a = item("a", "A"); a.directory_name = "1.21.1".into();
        library.instances = vec![a, item("b", "B")]; library.selected_id = Some("a".into());
        let library = store.save(library).unwrap();
        assert!(delete(&store, &root, "a", library.revision).unwrap_err().contains("share"));
        assert!(root.join("outside/keep").exists());
        fs::remove_dir_all(root).unwrap();
    }
}
