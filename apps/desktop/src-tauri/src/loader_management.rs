//! Recoverable loader-only updates. User content is never part of the transaction.
use crate::{install_commands::{Installer, Task}, storage_commands::Backend};
use launcher_core::{Library, ModLoader, storage::{atomic_write, LIBRARY}};
use launcher_install::{minecraft, safe_path, Progress};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::{Arc, atomic::Ordering}};
const JOURNAL: &str = "launcher-loader-transaction.json";
#[derive(Serialize, Deserialize)]
struct Journal { directory: String, manifest: Vec<u8>, receipt: Vec<u8>, library: Vec<u8> }
fn error(e: impl std::fmt::Display) -> String { e.to_string() }
fn paths(root: &Path, directory: &str) -> Result<[std::path::PathBuf;3],String> {
    minecraft::instance_directory(root,directory)?;
    Ok([safe_path(root,&format!("versions/{directory}/{directory}.json"))?,safe_path(root,&format!("versions/{directory}/launcher-install.json"))?,safe_path(root.parent().ok_or("Invalid game directory")?,LIBRARY)?])
}
pub fn recover(root: &Path) -> Result<(),String> {
    let path = safe_path(root,JOURNAL)?;
    if !path.exists() {return Ok(());}
    let journal: Journal = serde_json::from_slice(&fs::read(&path).map_err(error)?).map_err(error)?;
    let targets = paths(root,&journal.directory)?;
    for (target,bytes) in targets.iter().zip([&journal.manifest,&journal.receipt,&journal.library]) {atomic_write(target,bytes)?;}
    fs::remove_file(path).map_err(error)
}
fn commit(root: &Path, directory: &str, manifest: &[u8], receipt: &[u8], next: Library) -> Result<Library,String> {
    let targets = paths(root,directory)?;
    let journal = Journal {directory:directory.into(),manifest:fs::read(&targets[0]).map_err(error)?,receipt:fs::read(&targets[1]).map_err(error)?,library:fs::read(&targets[2]).map_err(error)?};
    let path = safe_path(root,JOURNAL)?;
    atomic_write(&path,&serde_json::to_vec(&journal).map_err(error)?)?;
    let result = (|| {
        atomic_write(&targets[0],manifest)?;
        atomic_write(&targets[1],receipt)?;
        let library = launcher_core::Store::new(targets[2].clone()).save(next)?;
        fs::remove_file(&path).map_err(error)?;
        Ok(library)
    })();
    if result.is_err() { recover(root)?; }
    result
}
#[tauri::command]
pub async fn change_mod_loader(backend: tauri::State<'_,Arc<Backend>>, installer: tauri::State<'_,Installer>, instance_id:String, revision:u64, loader:ModLoader) -> Result<Library,String> {
    launcher_install::require_platform()?;
    loader.validate()?;
    let (root,mut library,item) = {
        let guard=backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
        backend.ensure_idle()?;
        let storage=guard.as_ref().map_err(Clone::clone)?;
        let library=storage.store().load()?;
        if library.revision != revision {return Err("The instance library changed; reload and retry".into());}
        let item=library.instances.iter().find(|i|i.id==instance_id).ok_or("Instance not found; reload and retry")?.clone();
        if item.loader.as_ref().is_some_and(|old|old.kind!=loader.kind) {return Err("Switching mod loader families is not supported".into());}
        if item.loader.as_ref()==Some(&loader) {return Ok(library);}
        backend.busy.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).map_err(|_| "Another file task is running")?;
        (storage.location.root.clone(),library,item)
    };
    let backend=backend.inner().clone();
    let task=installer.task.clone(); let cancel=installer.cancel.clone();
    cancel.store(false,Ordering::SeqCst);
    match task.lock() {
        Ok(mut t)=> *t=Task {running:true,version:format!("{} {}",loader.kind,loader.version),progress:Progress{phase:"Downloading and verifying mod loader".into(),..Default::default()},..Default::default()},
        Err(_)=>{backend.busy.store(false,Ordering::SeqCst);return Err("Installation task state is unavailable".into());}
    }
    crate::logs::record("Mod loader change requested");
    tauri::async_runtime::spawn_blocking(move || {
        struct Release(Arc<Backend>);
        impl Drop for Release {fn drop(&mut self){self.0.busy.store(false,Ordering::SeqCst);}}
        let _release=Release(backend.clone());
        let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (manifest,receipt)=minecraft::prepare_loader_change(&root,&item,&loader,&cancel,&|p|{if let Ok(mut t)=task.lock(){t.progress=p;}})?;
            launcher_install::check_cancel(&cancel)?;
            library.instances.iter_mut().find(|i|i.id==instance_id).ok_or("Instance not found; reload and retry")?.loader=Some(loader);
            let mut guard=backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
            let result=commit(&root,item.directory(),&manifest,&serde_json::to_vec_pretty(&receipt).map_err(error)?,library);
            if result.is_err() && root.join(JOURNAL).exists() { *guard=Err("Mod loader recovery failed; restart the launcher to retry".into()); }
            result
        })).unwrap_or_else(|_|Err("Mod loader update stopped unexpectedly".into()));
        // Also recover after a panic before allowing any other file operation.
        if root.join(JOURNAL).exists() {
            if let Err(e)=recover(&root) {if let Ok(mut guard)=backend.storage.lock(){*guard=Err(e);}}
        }
        crate::logs::record(if result.is_ok() {"Mod loader change completed"} else {"Mod loader change failed; see instance log"});
        if let Ok(mut t)=task.lock(){t.running=false;t.cancelling=false; if let Err(e)=&result {t.error=Some(e.clone());} else {t.progress.phase="Installation complete".into();}}
        result
    }).await.map_err(error)?
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn failed_commit_restores_all_three_files() {
        let root=std::env::temp_dir().join(format!("loader-commit-{}",std::process::id())).join(".minecraft");
        fs::create_dir_all(root.join("versions/test")).unwrap();
        let targets=paths(&root,"test").unwrap();
        atomic_write(&targets[0],b"old manifest").unwrap();
        atomic_write(&targets[1],b"old receipt").unwrap();
        let store=launcher_core::Store::new(targets[2].clone());
        let library=store.save(Library::default()).unwrap();
        let original=fs::read(&targets[2]).unwrap();
        let mut stale=library.clone(); stale.revision=0;
        assert!(commit(&root,"test",b"new manifest",b"new receipt",stale).is_err());
        assert_eq!(fs::read(&targets[0]).unwrap(),b"old manifest");
        assert_eq!(fs::read(&targets[1]).unwrap(),b"old receipt");
        assert_eq!(fs::read(&targets[2]).unwrap(),original);
        assert!(!root.join(JOURNAL).exists());
        let next=commit(&root,"test",b"new manifest",b"new receipt",library).unwrap();
        assert_eq!(next.revision,2);
        assert_eq!(fs::read(&targets[0]).unwrap(),b"new manifest");
        recover(&root).unwrap();
        assert_eq!(fs::read(&targets[0]).unwrap(),b"new manifest");
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }
    #[test] fn interrupted_commit_restores_only_metadata() {
        let root=std::env::temp_dir().join(format!("loader-recovery-{}",std::process::id())).join(".minecraft");
        fs::create_dir_all(root.join("versions/test/saves")).unwrap();
        fs::write(root.join("versions/test/saves/world"),b"keep").unwrap();
        let journal=Journal{directory:"test".into(),manifest:b"old manifest".to_vec(),receipt:b"old receipt".to_vec(),library:b"old library".to_vec()};
        atomic_write(&root.join(JOURNAL),&serde_json::to_vec(&journal).unwrap()).unwrap();
        atomic_write(&root.join("versions/test/test.json"),b"new manifest").unwrap();
        recover(&root).unwrap(); recover(&root).unwrap();
        assert_eq!(fs::read(root.join("versions/test/test.json")).unwrap(),b"old manifest");
        assert_eq!(fs::read(root.parent().unwrap().join(LIBRARY)).unwrap(),b"old library");
        assert_eq!(fs::read(root.join("versions/test/saves/world")).unwrap(),b"keep");
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }
}
