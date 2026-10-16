//! Scoped local log access. The renderer supplies an instance ID and a file name,
//! never an arbitrary filesystem path. Log contents are untrusted plain text.
use crate::storage_commands::Backend;
use launcher_install::safe_path;
use serde::Serialize;
use std::{fs, io::{Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, sync::{Arc, Mutex, OnceLock}};
const VIEW_LIMIT: u64 = 512 * 1024;
const SESSION_LIMIT: u64 = 5 * 1024 * 1024;
static SESSION: OnceLock<Mutex<fs::File>> = OnceLock::new();
fn error(e: impl std::fmt::Display) -> String { e.to_string() }
pub fn initialize(home: &Path) -> Result<(), String> {
    let root=safe_path(home,"logs")?;
    fs::create_dir_all(&root).map_err(error)?;
    let stamp=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(error)?.as_nanos();
    let file=fs::OpenOptions::new().create_new(true).write(true).open(root.join(format!("launcher-{stamp}.log"))).map_err(error)?;
    let _=SESSION.set(Mutex::new(file));
    // Retain only launcher-owned sessions; game and loader logs are never pruned here.
    let mut old=fs::read_dir(&root).map_err(error)?.filter_map(Result::ok)
        .filter(|e|e.file_type().is_ok_and(|t|t.is_file()) && e.file_name().to_string_lossy().starts_with("launcher-") && e.file_name().to_string_lossy().ends_with(".log")).collect::<Vec<_>>();
    old.sort_by_key(|e|std::cmp::Reverse(e.file_name()));
    for entry in old.into_iter().skip(10) { let _=fs::remove_file(entry.path()); }
    record("Launcher session started");
    Ok(())
}
pub fn record(message: &str) {
    let Some(file)=SESSION.get() else {return};
    let Ok(mut file)=file.lock() else {return};
    if file.metadata().is_ok_and(|m|m.len() >= SESSION_LIMIT) {return}
    let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    // Do not record credentials or launch arguments. Events contain operation status only.
    let _=writeln!(file,"[{now}] {message}");
    let _=file.flush();
}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct LogEntry { name: String, modified: u64, size: u64 }
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct LogContent { text: String, truncated: bool }
fn scope(_app: &tauri::AppHandle, backend: &Backend, id: Option<&str>) -> Result<PathBuf,String> {
    if let Some(id)=id {
        let guard=backend.storage.lock().map_err(|_|"Instance storage is unavailable")?;
        let storage=guard.as_ref().map_err(Clone::clone)?;
        let library=storage.store().load()?;
        let item=library.instances.iter().find(|i|i.id==id).ok_or("Instance not found; reload and retry")?;
        launcher_install::minecraft::instance_directory(&storage.location.root,item.directory())
    } else { crate::storage_commands::launcher_directory().map_err(error) }
}
fn log_path(root: &Path, name: &str, instance: bool) -> Result<PathBuf,String> {
    let parts=name.split('/').collect::<Vec<_>>();
    if parts.len()!=2 || !(parts[0]=="logs" || instance && parts[0]=="crash-reports") ||
        ![".log",".log.gz",".txt"].iter().any(|suffix|parts[1].ends_with(suffix)) {
        return Err("Invalid log file selection".into());
    }
    let path=safe_path(root,name)?;
    if !path.is_file() { return Err("Log file is no longer available; refresh the list".into()); }
    Ok(path)
}
fn list(root: &Path, instance: bool) -> Result<Vec<LogEntry>,String> {
    let mut result=vec![];
    for folder in ["logs","crash-reports"] {
        if !instance && folder!="logs" {continue}
        let dir=safe_path(root,folder)?;
        let entries=match fs::read_dir(dir) {Ok(entries)=>entries,Err(e) if e.kind()==std::io::ErrorKind::NotFound=>continue,Err(e)=>return Err(error(e))};
        for entry in entries {
            let entry=entry.map_err(error)?;
            if !entry.file_type().map_err(error)?.is_file() {continue}
            let name=format!("{folder}/{}",entry.file_name().to_string_lossy());
            if log_path(root,&name,instance).is_err() {continue}
            let meta=entry.metadata().map_err(error)?;
            result.push(LogEntry {name,size:meta.len(),modified:meta.modified().ok().and_then(|t|t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d|d.as_millis() as u64).unwrap_or(0)});
        }
    }
    result.sort_by(|a,b|b.modified.cmp(&a.modified).then(b.name.cmp(&a.name)));
    result.truncate(200);
    Ok(result)
}
fn read(root: &Path, name: &str, instance: bool) -> Result<LogContent,String> {
    let path=log_path(root,name,instance)?;
    let mut file=fs::File::open(path).map_err(error)?;
    let mut bytes=vec![];
    let truncated;
    if name.ends_with(".gz") {
        flate2::read::GzDecoder::new(file).take(VIEW_LIMIT+1).read_to_end(&mut bytes).map_err(error)?;
        truncated=bytes.len() as u64>VIEW_LIMIT;
        bytes.truncate(VIEW_LIMIT as usize);
    } else {
        let size=file.metadata().map_err(error)?.len();
        truncated=size>VIEW_LIMIT;
        file.seek(SeekFrom::Start(size.saturating_sub(VIEW_LIMIT))).map_err(error)?;
        file.take(VIEW_LIMIT).read_to_end(&mut bytes).map_err(error)?;
    }
    Ok(LogContent {text:String::from_utf8_lossy(&bytes).into_owned(),truncated})
}
#[tauri::command]
pub async fn list_logs(app: tauri::AppHandle, backend: tauri::State<'_,Arc<Backend>>, instance_id: Option<String>) -> Result<Vec<LogEntry>,String> {
    let root=scope(&app,&backend,instance_id.as_deref())?;
    tauri::async_runtime::spawn_blocking(move||list(&root,instance_id.is_some())).await.map_err(error)?
}
#[tauri::command]
pub async fn read_log(app: tauri::AppHandle, backend: tauri::State<'_,Arc<Backend>>, instance_id: Option<String>, name:String) -> Result<LogContent,String> {
    let root=scope(&app,&backend,instance_id.as_deref())?;
    tauri::async_runtime::spawn_blocking(move||read(&root,&name,instance_id.is_some())).await.map_err(error)?
}
#[tauri::command]
pub async fn open_log_location(app: tauri::AppHandle, backend: tauri::State<'_,Arc<Backend>>, instance_id: Option<String>, name: Option<String>) -> Result<(),String> {
    let root=scope(&app,&backend,instance_id.as_deref())?;
    let path=match name {Some(name)=>log_path(&root,&name,instance_id.is_some())?,None=>if instance_id.is_some(){root}else{safe_path(&root,"logs")?}};
    tauri::async_runtime::spawn_blocking(move||{
        if !path.exists(){return Err("Log directory is not available yet".into());}
        launcher_platform::open_external(path)
    }).await.map_err(error)?
}
#[cfg(test)] mod tests {
    use super::*;
    fn root()->PathBuf {
        let p=std::env::temp_dir().join(format!("log-tests-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(p.join("logs")).unwrap();p
    }
    #[test] fn files_are_scoped_and_lists_include_game_and_installer_logs() {
        let root=root();fs::write(root.join("logs/latest.log"),b"game").unwrap();fs::write(root.join("logs/forge-install.log"),b"installer").unwrap();
        fs::create_dir(root.join("crash-reports")).unwrap();fs::write(root.join("crash-reports/crash.txt"),b"crash").unwrap();
        assert_eq!(list(&root,true).unwrap().len(),3);assert_eq!(list(&root,false).unwrap().len(),2);
        for name in ["../secret.log","logs/../../secret.log","logs/a/../latest.log","logs/file.json","/etc/passwd"] {assert!(read(&root,name,true).is_err());}
        #[cfg(unix)] {std::os::unix::fs::symlink("/etc/passwd",root.join("logs/link.log")).unwrap();assert!(read(&root,"logs/link.log",true).is_err());}
        assert_eq!(read(&root,"logs/latest.log",true).unwrap().text,"game");fs::remove_dir_all(root).unwrap();
    }
    #[test] fn reads_are_bounded_and_compressed_logs_are_supported() {
        let root=root();let mut data=vec![b'a';VIEW_LIMIT as usize+20];data.extend_from_slice(b"tail");
        fs::write(root.join("logs/large.log"),&data).unwrap();
        let content=read(&root,"logs/large.log",true).unwrap();assert!(content.truncated);assert!(content.text.ends_with("tail"));assert_eq!(content.text.len(),VIEW_LIMIT as usize);
        let mut gzip=flate2::write::GzEncoder::new(fs::File::create(root.join("logs/old.log.gz")).unwrap(),flate2::Compression::default());
        gzip.write_all(b"archived").unwrap();gzip.finish().unwrap();
        assert_eq!(read(&root,"logs/old.log.gz",true).unwrap().text,"archived");fs::remove_dir_all(root).unwrap();
    }
}
