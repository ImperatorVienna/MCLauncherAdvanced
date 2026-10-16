use crate::{accounts::AccountStore, storage_commands::Backend, launch_plan};
use serde::Serialize;
use std::{sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, process::{Command, Stdio}};
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status { running: bool, preparing: bool, instance_id: Option<String>, error: Option<String>, log_path: Option<String> }
#[derive(Default)]
pub struct GameLauncher(pub Arc<Mutex<Status>>, pub Arc<AtomicBool>);
impl GameLauncher {
    // Keep the session lock alive while the child uses the instance files.
    pub fn defer_close(&self, window: &tauri::WebviewWindow) -> Result<bool, String> {
        let state = self.0.lock().map_err(|_| "Game state is unavailable")?;
        if !state.running { return Ok(false); }
        window.hide().map_err(|e| e.to_string())?;
        self.1.store(true, Ordering::SeqCst);
        Ok(true)
    }
}
#[tauri::command]
pub fn game_status(launcher: tauri::State<'_, GameLauncher>) -> Result<Status, String> {
    Ok(launcher.0.lock().map_err(|_| "Game state is unavailable")?.clone())
}
#[tauri::command]
pub fn launch_game(app: tauri::AppHandle, backend: tauri::State<'_, Arc<Backend>>, accounts: tauri::State<'_, AccountStore>, launcher: tauri::State<'_, GameLauncher>, installer: tauri::State<'_, crate::install_commands::Installer>, instance_id: String) -> Result<(), String> {
    launcher_install::require_platform()?;
    let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    let storage = guard.as_ref().map_err(Clone::clone)?;
    let item = storage.store().load()?.instances.into_iter().find(|i| i.id == instance_id).ok_or("Instance not found; reload and retry")?;
    let selected_account = accounts.load()?.selected.ok_or("Select an account before launching")?;
    let account_store = accounts.inner().clone();
    let root = storage.location.root.clone();
    if backend.busy.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() { return Err("Another file task or game is running".into()); }
    let status = launcher.0.clone();
    match status.lock() {
        Ok(mut state) => *state = Status { running: true, preparing: true, instance_id: Some(instance_id), ..Default::default() },
        Err(_) => { backend.busy.store(false, Ordering::SeqCst); return Err("Game state is unavailable".into()); }
    }
    let task = installer.task.clone();
    let cancel = installer.cancel.clone();
    cancel.store(false, Ordering::SeqCst);
    let owner = backend.inner().clone();
    let close_requested = launcher.1.clone();
    crate::logs::record("Game launch requested");
    let thread = std::thread::Builder::new().name("minecraft-process".into()).spawn(move || {
        struct Release(Arc<Backend>);
        impl Drop for Release { fn drop(&mut self) { self.0.busy.store(false, Ordering::SeqCst); } }
        let _release = Release(owner);
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let java = item.java_path.as_ref().ok_or("Select Java in instance management before launching")?;
            if launcher_install::java::probe(std::path::Path::new(java), "selected", &AtomicBool::new(false))?.is_none() { return Err("The selected Java cannot run; select another runtime".into()); }
            if !launcher_install::minecraft::installed_with_loader(&root, &item.minecraft_version, item.directory(), item.loader.as_ref()).installed {
                *task.lock().map_err(|_| "Installation task state is unavailable")? = crate::install_commands::Task {running:true,version:item.name.clone(),..Default::default()};
                launcher_install::minecraft::install_with_loader(&root, Some(std::path::Path::new(java)), &item.minecraft_version, item.directory(), item.loader.as_ref(), &cancel, &|progress| {
                    if let Ok(mut task)=task.lock() {task.progress=progress;}
                })?;
                if let Ok(mut task)=task.lock() {task.running=false;}
            }
            launcher_install::check_cancel(&cancel)?;
            let available = crate::memory::available_memory_bytes()?;
            item.launch_settings.validate_available_memory(available)?;
            let memory = item.launch_settings.max_memory_mib.map(u64::from).unwrap_or((available / 1024 / 1024 / 2).min(4096));
            if memory < 103 { return Err("Not enough available memory to launch the game".into()); }
            let file = launcher_install::safe_path(&root, &format!("versions/{0}/{0}.json", item.directory()))?;
            let metadata = serde_json::from_slice(&std::fs::read(file).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
            let kernel = launcher_platform::kernel_version().map_err(|e| e.to_string())?;
            let identity = crate::microsoft_auth::identity(&account_store, &selected_account)?;
            let plan = launch_plan::build_authenticated(&root, &item, &metadata, &identity, memory, kernel.trim())?;
            launch_plan::apply_display(&plan.directory, &item.launch_settings.display_mode)?;
            let logs = launcher_install::safe_path(&plan.directory, "logs")?;
            std::fs::create_dir_all(&logs).map_err(|e| e.to_string())?;
            let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos();
            let log_path = logs.join(format!("launcher-{stamp}.log"));
            let output = std::fs::OpenOptions::new().write(true).create_new(true).open(&log_path).map_err(|e| e.to_string())?;
            if let Ok(mut state) = status.lock() { state.log_path = Some(log_path.to_string_lossy().into_owned()); }
            let mut child = spawn_java(java, &plan, output)?;
            if let Ok(mut state) = status.lock() { state.preparing = false; }
            let exit = child.wait().map_err(|e| format!("Failed to monitor the game: {e}"))?;
            if !exit.success() { return Err(format!("Game exited abnormally: {exit}")); }
            Ok(())
        })).unwrap_or_else(|_| Err("Game launch stopped unexpectedly".into()));
        if let Ok(mut task)=task.lock() {task.running=false; task.cancelling=false;}
        crate::logs::record(if outcome.is_ok() {"Game session ended normally"} else {"Game launch or session failed; see instance log"});
        if let Ok(mut state) = status.lock() { state.running = false; state.preparing = false; state.error = outcome.err(); }
        if close_requested.load(Ordering::SeqCst) { app.exit(0); }
    });
    if let Err(e) = thread {
        backend.busy.store(false, Ordering::SeqCst);
        if let Ok(mut state) = launcher.0.lock() { state.running = false; state.preparing = false; }
        return Err(e.to_string());
    }
    Ok(())
}

/// A single subprocess path is shared by production and the JVM smoke test.
pub(crate) fn spawn_java(java: &str, plan: &launch_plan::Plan, output: std::fs::File) -> Result<std::process::Child, String> {
    launcher_platform::clean_child_environment(Command::new(java).args(&plan.args).current_dir(&plan.directory))
        .stdin(Stdio::null()).stdout(output.try_clone().map_err(|e| e.to_string())?).stderr(output)
        .spawn().map_err(|e| format!("Failed to start Java: {e}"))
}
