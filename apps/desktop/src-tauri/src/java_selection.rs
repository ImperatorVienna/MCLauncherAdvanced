//! Startup validation never substitutes a different runtime for a player's choice.
use launcher_core::storage::Storage;
use std::{path::Path, sync::atomic::AtomicBool};
pub fn reconcile(storage: &Storage) -> Result<(), String> {
    let store = storage.store();
    let mut library = store.load()?;
    let cancel = AtomicBool::new(false);
    let mut checked = std::collections::HashMap::new();
    let mut changed = false;
    for item in &mut library.instances {
        let Some(path) = item.java_path.clone() else {continue};
        let result = checked.entry(path.clone()).or_insert_with(||launcher_install::java::probe(Path::new(&path), "selected", &cancel));
        // An infrastructure error must not discard a selection. A completed negative probe can.
        if matches!(result, Ok(None)) { item.java_path = None; changed = true; }
    }
    if changed { store.save(library)?; }
    Ok(())
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn absent_java_is_cleared_without_selecting_a_replacement() {
        let root=std::env::temp_dir().join(format!("java-reconcile-{}",std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let storage=Storage::open(&root).unwrap();
        let item=launcher_core::Instance {id:"test".into(),name:"test".into(),directory_name:"test".into(),minecraft_version:"1.21.1".into(),icon:"grass".into(),loader:Some(launcher_core::ModLoader {kind:"neoforge".into(),version:"21.1.1".into()}),java_path:Some(root.join("absent-java").to_string_lossy().into()),launch_settings:Default::default()};
        storage.store().save(launcher_core::Library {instances:vec![item],selected_id:Some("test".into()),..Default::default()}).unwrap();
        reconcile(&storage).unwrap();
        assert!(storage.store().load().unwrap().instances[0].java_path.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }
}
