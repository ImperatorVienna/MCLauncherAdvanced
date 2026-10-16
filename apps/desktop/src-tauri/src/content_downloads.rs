//! Resolve instance context on the backend; the renderer never chooses paths or compatibility.
use crate::{storage_commands::Backend,install_commands::{Installer,Task}};
use launcher_install::{content::Kind,content_source::{self,Provider,Search,Updates,Update},Progress};
use std::sync::{Arc,atomic::Ordering};
fn context(backend:&Backend,id:&str)->Result<(std::path::PathBuf,launcher_core::Instance),String> {
    let guard=backend.storage.lock().map_err(|_|"Instance storage is unavailable")?;
    let storage=guard.as_ref().map_err(Clone::clone)?;
    let item=storage.store().load()?.instances.into_iter().find(|i|i.id==id).ok_or("Instance not found; reload and retry")?;
    Ok((launcher_install::minecraft::instance_directory(&storage.location.root,item.directory())?,item))
}
#[tauri::command]
pub async fn identify_instance_content(backend:tauri::State<'_,Arc<Backend>>,instance_id:String,kind:Kind)->Result<Vec<content_source::ContentLabel>,String> {
    let (root,_)=context(&backend,&instance_id)?;
    tauri::async_runtime::spawn_blocking(move||content_source::labels(&root,kind)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn search_instance_content(backend:tauri::State<'_,Arc<Backend>>,instance_id:String,kind:Kind,query:String,offset:u32)->Result<Search,String> {
    let (_,item)=context(&backend,&instance_id)?;
    tauri::async_runtime::spawn_blocking(move||Provider::new()?.search(&item,kind,&query,offset)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn check_content_updates(backend:tauri::State<'_,Arc<Backend>>,instance_id:String,kind:Kind)->Result<Updates,String> {
    let (root,item)=context(&backend,&instance_id)?;
    tauri::async_runtime::spawn_blocking(move||content_source::updates(&root,&item,kind)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn validate_content_updates(backend:tauri::State<'_,Arc<Backend>>,instance_id:String,kind:Kind,updates:Vec<Update>)->Result<Vec<Update>,String> {
    let (root,_)=context(&backend,&instance_id)?;
    tauri::async_runtime::spawn_blocking(move||content_source::retain_updates(&root,kind,updates)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub async fn catalog_content_states(backend:tauri::State<'_,Arc<Backend>>,instance_id:String,kind:Kind,projects:Vec<String>)->Result<Vec<content_source::CatalogState>,String> {
    let (root,item)=context(&backend,&instance_id)?;
    tauri::async_runtime::spawn_blocking(move||content_source::catalog_states(&root,&item,kind,&projects)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn start_content_installation(backend:tauri::State<'_,Arc<Backend>>,installer:tauri::State<'_,Installer>,instance_id:String,kind:Kind,project:String,update:Option<Update>)->Result<(),String> {
    let guard=backend.storage.lock().map_err(|_|"Instance storage is unavailable")?;
    let storage=guard.as_ref().map_err(Clone::clone)?;
    let item=storage.store().load()?.instances.into_iter().find(|i|i.id==instance_id).ok_or("Instance not found; reload and retry")?;
    let root=launcher_install::minecraft::instance_directory(&storage.location.root,item.directory())?;
    if backend.busy.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).is_err(){return Err("Another file task is running".into());}
    let task=installer.task.clone();
    let cancel=installer.cancel.clone();
    cancel.store(false,Ordering::SeqCst);
    match task.lock() {
        Ok(mut t)=>*t=Task{content:true,running:true,version:item.name.clone(),progress:Progress{phase:"Preparing content installation".into(),..Default::default()},..Default::default()},
        Err(_)=>{backend.busy.store(false,Ordering::SeqCst);return Err("Installation task state is unavailable".into());}
    }
    let owner=backend.inner().clone();
    let result=std::thread::Builder::new().name("content-install".into()).spawn(move||{
        struct Release(Arc<Backend>);
        impl Drop for Release {fn drop(&mut self){self.0.busy.store(false,Ordering::SeqCst);}}
        let _release=Release(owner);
        let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||content_source::install(&root,&item,kind,&project,update,&cancel,&|progress|{
            if let Ok(mut t)=task.lock(){t.progress=progress;}
        }))).unwrap_or_else(|_|Err("Content installation stopped unexpectedly".into()));
        if let Ok(mut t)=task.lock() {
            t.running=false;t.cancelling=false;
            match outcome {Ok(())=>t.progress.phase="Installation complete".into(),Err(error)=>t.error=Some(error)}
        }
    });
    if let Err(error)=result {
        backend.busy.store(false,Ordering::SeqCst);
        if let Ok(mut t)=installer.task.lock(){t.running=false;t.error=Some(error.to_string());}
        return Err(error.to_string());
    }
    Ok(())
}
