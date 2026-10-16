//! Serialized, instance-scoped content operations; no renderer-supplied paths.
use crate::storage_commands::Backend;
use launcher_install::content::{self, Kind, Listing};
use std::{path::Path, sync::Arc};
fn with_instance<T>(backend: &Backend, id: &str, write: bool, action: impl FnOnce(&Path)->Result<T,String>) -> Result<T,String> {
    let guard=backend.storage.lock().map_err(|_|"Instance storage is unavailable")?;
    if write {backend.ensure_idle()?;}
    let storage=guard.as_ref().map_err(Clone::clone)?;
    let library=storage.store().load()?;
    let instance=library.instances.iter().find(|i|i.id==id).ok_or("Instance not found; reload and retry")?;
    let directory=launcher_install::minecraft::instance_directory(&storage.location.root,instance.directory())?;
    action(&directory)
}
#[tauri::command]
pub fn delete_instance_content(backend:tauri::State<'_,Arc<Backend>>, instance_id:String, kind:Kind, name:String, enabled:bool) -> Result<Listing,String> {
    with_instance(&backend,&instance_id,true,|root|content::delete(root,kind,&name,enabled))
}
#[tauri::command]
pub fn scan_instance_content(backend:tauri::State<'_,Arc<Backend>>, instance_id:String, kind:Kind) -> Result<Listing,String> {
    with_instance(&backend,&instance_id,false,|root|content::scan(root,kind,backend.ensure_idle().is_ok()))
}
#[tauri::command]
pub fn set_instance_content_enabled(backend:tauri::State<'_,Arc<Backend>>, instance_id:String, kind:Kind, name:String, enabled:bool) -> Result<Listing,String> {
    with_instance(&backend,&instance_id,true,|root|content::set_enabled(root,kind,&name,enabled))
}
#[tauri::command]
pub fn open_instance_content_directory(backend:tauri::State<'_,Arc<Backend>>, instance_id:String, kind:Kind) -> Result<(),String> {
    with_instance(&backend,&instance_id,true,|root| {
        let listing=content::scan(root,kind,true)?;
        launcher_platform::open_external(&listing.directory)?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_and_busy_guard_prevent_cross_instance_writes() {
        let home=std::env::temp_dir().join(format!("content-ipc-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let backend=Backend::for_test(home.clone());
        {
            let guard=backend.storage.lock().unwrap();
            let store=guard.as_ref().unwrap().store();
            let mut library=store.load().unwrap();
            for id in ["first","second"] {
                library.instances.push(launcher_core::Instance{id:id.into(),name:id.into(),minecraft_version:"1.21.1".into(),icon:"grass".into(),loader:None,java_path:None,directory_name:id.into(),launch_settings:Default::default()});
            }
            library.selected_id=Some("first".into());
            store.save(library).unwrap();
        }
        with_instance(&backend,"first",true,|root|content::scan(root,Kind::Mods,true)).unwrap();
        assert!(home.join(".minecraft/versions/first/mods").exists());
        assert!(!home.join(".minecraft/versions/second/mods").exists());
        assert!(with_instance(&backend,"missing",true,|_|Ok(())).is_err());
        backend.busy.store(true,std::sync::atomic::Ordering::SeqCst);
        assert!(with_instance(&backend,"second",true,|root|content::scan(root,Kind::Mods,true)).is_err());
        assert!(!home.join(".minecraft/versions/second/mods").exists());
        std::fs::remove_dir_all(home).unwrap();
    }
}
