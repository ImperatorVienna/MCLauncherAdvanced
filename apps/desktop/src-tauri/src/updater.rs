//! Application lifecycle and updater IPC. Domain services own release/download rules.
use crate::storage_commands::Backend;
use launcher_install::{download, self_update::{self, Update}};
use serde::Serialize;
use std::sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}};

#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status {
    phase: String,
    version: Option<String>,
    completed: u64,
    total: u64,
    error: Option<String>,
}
impl Default for Status {
    fn default()->Self { Self {phase:"idle".into(),version:None,completed:0,total:0,error:None} }
}
#[derive(Default)]
struct Inner { status: Status, available: Option<Update>, staged: Option<tempfile::TempDir> }
#[derive(Default)]
pub struct Updater { inner:Mutex<Inner>, active:AtomicBool, cancel:AtomicBool }
struct Active(Arc<Updater>);
impl Drop for Active { fn drop(&mut self){self.0.active.store(false,Ordering::SeqCst);} }
struct Busy(Arc<Backend>);
impl Drop for Busy { fn drop(&mut self){self.0.busy.store(false,Ordering::SeqCst);} }
impl Updater {
    fn failure(&self,e:String){
        crate::logs::record(&format!("Launcher update: {e}"));
        let mut inner=self.inner.lock().unwrap();inner.status.phase="error".into();inner.status.error=Some(e);
    }
    pub fn check(self:&Arc<Self>,current:String)->Result<(),String>{
        self.active.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).map_err(|_|"An update operation is already running")?;
        { let mut i=self.inner.lock().unwrap(); i.status=Status {phase:"checking".into(),..Status::default()};i.available=None;i.staged=None; }
        let this=self.clone();
        std::thread::spawn(move||{
            let _active=Active(this.clone());
            let result=(||{
                let (target,_)=launcher_platform::updater::target()?;
                self_update::check(&current,target)
            })();
            match result {
                Ok((published,update))=>{
                    let mut i=this.inner.lock().unwrap();
                    i.status.phase=if update.is_some(){"available"}else if published{"current"}else{"unpublished"}.into();
                    i.status.version=update.as_ref().map(|u|u.version.clone());i.available=update;
                },
                Err(e)=>this.failure(e),
            }
        });
        Ok(())
    }
}
#[tauri::command]
pub fn launcher_update_status(updater:tauri::State<'_,Arc<Updater>>)->Status { updater.inner.lock().unwrap().status.clone() }
#[tauri::command]
pub fn check_launcher_update(app:tauri::AppHandle,updater:tauri::State<'_,Arc<Updater>>)->Result<(),String>{updater.check(app.package_info().version.to_string())}
#[tauri::command]
pub fn cancel_launcher_update(updater:tauri::State<'_,Arc<Updater>>) { updater.cancel.store(true,Ordering::SeqCst); }
#[tauri::command]
pub fn download_launcher_update(updater:tauri::State<'_,Arc<Updater>>,backend:tauri::State<'_,Arc<Backend>>)->Result<(),String>{
    updater.active.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).map_err(|_|"An update operation is already running")?;
    let active=Active(updater.inner().clone());
    backend.busy.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).map_err(|_|"A file task is running; wait for completion")?;
    let busy=Busy(backend.inner().clone());
    let update=updater.inner.lock().unwrap().available.clone().ok_or("Check for launcher updates first")?;
    updater.cancel.store(false,Ordering::SeqCst);
    {let mut i=updater.inner.lock().unwrap();i.status.phase="downloading".into();i.status.error=None;i.status.completed=0;i.status.total=update.file.size;}
    let this=updater.inner().clone();
    std::thread::spawn(move||{
        let _active=active;let _busy=busy;
        let result=(||{
            let (_,destination)=launcher_platform::updater::target()?;
            let root=destination.parent().ok_or("Invalid executable directory")?;
            let staging=tempfile::Builder::new().prefix(".launcher-update-").tempdir_in(root).map_err(|e|e.to_string())?;
            download::download_all(&download::large_file_client()?,staging.path(),std::slice::from_ref(&update.file),&this.cancel,&|p|{
                let mut i=this.inner.lock().unwrap();i.status.completed=p.completed;i.status.total=p.total;
            },"Downloading launcher update")?;
            Ok::<_,String>(staging)
        })();
        match result {
            Ok(staging)=>{let mut i=this.inner.lock().unwrap();i.staged=Some(staging);i.status.phase="ready".into();},
            Err(e)=>this.failure(e),
        }
    });
    Ok(())
}
#[tauri::command]
pub async fn apply_launcher_update(app:tauri::AppHandle,updater:tauri::State<'_,Arc<Updater>>,backend:tauri::State<'_,Arc<Backend>>)->Result<(),String>{
    let updater=updater.inner().clone();let backend=backend.inner().clone();
    tauri::async_runtime::spawn_blocking(move||apply(app,updater,backend)).await.map_err(|e|e.to_string())?
}
fn apply(app:tauri::AppHandle,updater:Arc<Updater>,backend:Arc<Backend>)->Result<(),String>{
    updater.active.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).map_err(|_|"An update operation is already running")?;
    let _active=Active(updater.clone());
    backend.busy.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).map_err(|_|"A file task is running; wait for completion")?;
    let busy=Busy(backend.clone());
    let result:Result<(),String>=(||{
        let mut i=updater.inner.lock().unwrap();
        let update=i.available.as_ref().ok_or("Check for launcher updates first")?;
        let directory=i.staged.as_ref().ok_or("Download the launcher update first")?;
        let staged=launcher_install::safe_path(directory.path(),&update.file.path)?;
        if !download::valid_file(&staged,&update.file,&AtomicBool::new(false))? {return Err("Downloaded file verification failed".into());}
        let (_,destination)=launcher_platform::updater::target()?;
        launcher_platform::updater::replace_after_exit(&destination,&staged)?;
        // Helper needs this directory after the launcher exits; retain its diagnostic log.
        let _=i.staged.take().unwrap().keep();
        i.status.phase="restarting".into();
        Ok(())
    })();
    if let Err(e)=result {updater.failure(e.clone());return Err(e);}
    // Keep all other writers blocked until process exit releases the session lock.
    std::mem::forget(busy);
    app.exit(0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn concurrent_checks_rejected_without_mutating_status(){
        let updater=Arc::new(Updater::default());updater.active.store(true,Ordering::SeqCst);
        updater.inner.lock().unwrap().status.phase="downloading".into();
        assert!(updater.check("1.0.0".into()).is_err());
        assert_eq!(updater.inner.lock().unwrap().status.phase,"downloading");
        drop(Active(updater.clone()));assert!(!updater.active.load(Ordering::SeqCst));
    }
    #[test]
    fn writer_guard_releases_on_failed_operation(){
        let directory=tempfile::tempdir().unwrap();
        let backend=Arc::new(Backend::for_test(directory.path().to_path_buf()));
        backend.busy.store(true,Ordering::SeqCst);
        drop(Busy(backend.clone()));assert!(backend.ensure_idle().is_ok());
    }
}
