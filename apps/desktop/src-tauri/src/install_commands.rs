use crate::storage_commands::Backend;
use launcher_install::{
    java::Java,
    minecraft::{self, Installed, Version},
    Progress,
};
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub content: bool,
    pub running: bool,
    pub cancelling: bool,
    pub version: String,
    pub progress: Progress,
    pub error: Option<String>,
    pub result: Option<Java>,
}
#[derive(Default)]
pub struct Installer {
    pub(crate) task: Arc<Mutex<Task>>,
    pub(crate) cancel: Arc<AtomicBool>,
}
#[tauri::command]
pub fn install_platform() -> launcher_install::Platform {
    launcher_install::platform()
}
#[tauri::command]
pub async fn list_minecraft_versions() -> Result<Vec<Version>, String> {
    tauri::async_runtime::spawn_blocking(minecraft::catalog)
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn installation_task(installer: tauri::State<'_, Installer>) -> Result<Task, String> {
    Ok(installer
        .task
        .lock()
        .map_err(|_| "Installation task state is unavailable")?
        .clone())
}
#[tauri::command]
pub fn cancel_installation(installer: tauri::State<'_, Installer>) -> Result<(), String> {
    let mut task = installer.task.lock().map_err(|_| "Installation task state is unavailable")?;
    if task.running {
        task.cancelling = true;
        installer.cancel.store(true, Ordering::SeqCst);
    }
    Ok(())
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceInstallStatus {
    instance_id: String,
    game_directory: std::path::PathBuf,
    #[serde(flatten)]
    status: Installed,
}
#[tauri::command]
pub async fn installed_versions(
    backend: tauri::State<'_, Arc<Backend>>,
) -> Result<Vec<InstanceInstallStatus>, String> {
    let (root, items) = {
        let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
        let storage = guard.as_ref().map_err(Clone::clone)?;
        (
            storage.location.root.clone(),
            storage.store().load()?.instances,
        )
    };
    tauri::async_runtime::spawn_blocking(move || {
        items
            .into_iter()
            .map(|item| {
                let game_directory = minecraft::instance_directory(&root, item.directory())?;
                let status = minecraft::installed_with_loader(&root, &item.minecraft_version, item.directory(), item.loader.as_ref());
                Ok(InstanceInstallStatus {
                    instance_id: item.id,
                    game_directory,
                    status,
                })
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn start_installation<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    backend: tauri::State<'_, Arc<Backend>>,
    installer: tauri::State<'_, Installer>,
    instance_id: String,
) -> Result<(), String> {
    launcher_install::require_platform()?;
    let _ = app;
    let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    let storage = guard.as_ref().map_err(Clone::clone)?;
    let item = storage
        .store()
        .load()?
        .instances
        .into_iter()
        .find(|i| i.id == instance_id)
        .ok_or("Instance not found; reload and retry")?;
    let root = storage.location.root.clone();
    minecraft::instance_directory(&root, item.directory())?;
    start_worker(backend.inner().clone(), &installer, root, item)
}
fn start_worker(backend: Arc<Backend>, installer: &Installer, root: std::path::PathBuf, item: launcher_core::Instance) -> Result<(), String> {
    if backend
        .busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another file task is running".into());
    }
    let task = installer.task.clone();
    let cancel = installer.cancel.clone();
    cancel.store(false, Ordering::SeqCst);
    match task.lock() {
        Ok(mut t) => {
            *t = Task {
                running: true,
                version: item.minecraft_version.clone(),
                progress: Progress {
                    phase: "Preparing installation".into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        Err(_) => {
            backend.busy.store(false, Ordering::SeqCst);
            return Err("Installation task state is unavailable".into());
        }
    }
    crate::logs::record("Game installation requested");
    let worker_backend = backend.clone();
    let result = std::thread::Builder::new()
        .name("minecraft-install".into())
        .spawn(move || {
            struct Release(Arc<Backend>);
            impl Drop for Release {
                fn drop(&mut self) {
                    self.0.busy.store(false, Ordering::SeqCst);
                }
            }
            let _release = Release(worker_backend);
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                minecraft::install_with_loader(
                    &root,
                    item.java_path.as_deref().map(std::path::Path::new),
                    &item.minecraft_version,
                    item.directory(),
                    item.loader.as_ref(),
                    &cancel,
                    &|p| {
                        if let Ok(mut t) = task.lock() {
                            t.progress = p;
                        }
                    },
                )
            }))
            .unwrap_or_else(|_| Err("Installation stopped unexpectedly; retry to repair".into()));
            crate::logs::record(if outcome.is_ok() {"Game installation completed"} else {"Game installation interrupted or failed; see instance details"});
            if let Ok(mut t) = task.lock() {
                t.running = false;
                t.cancelling = false;
                match outcome {
                    Ok(receipt) => {
                        t.result = receipt.java;
                        t.progress.phase = "Installation complete".into();
                    }
                    Err(e) => t.error = Some(e),
                }
            }
        });
    if let Err(e) = result {
        backend.busy.store(false, Ordering::SeqCst);
        if let Ok(mut t) = installer.task.lock() {
            t.running = false;
            t.error = Some(e.to_string());
        }
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub async fn list_java() -> Result<Vec<Java>, String> {
    let root = crate::storage_commands::launcher_directory().map_err(|e|e.to_string())?.join("runtimes/java");
    tauri::async_runtime::spawn_blocking(move || launcher_install::java::inventory(&root))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn recommended_java(version: String) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || minecraft::recommendation(&version))
        .await
        .map_err(|e| e.to_string())?
}
#[tauri::command]
pub fn start_java_installation(
    backend: tauri::State<'_, Arc<Backend>>,
    installer: tauri::State<'_, Installer>,
    major: u32,
) -> Result<(), String> {
    launcher_install::require_platform()?;
    if !(8..=99).contains(&major) {
        return Err("Invalid Java major version".into());
    }
    let root = crate::storage_commands::launcher_directory().map_err(|e|e.to_string())?.join("runtimes/java");
    let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    if backend
        .busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err("Another file task is running".into());
    }
    drop(guard);
    let task = installer.task.clone();
    let cancel = installer.cancel.clone();
    cancel.store(false, Ordering::SeqCst);
    match task.lock() {
        Ok(mut t) => {
            *t = Task {
                running: true,
                version: format!("Java {major}"),
                progress: Progress {
                    phase: "Preparing Java download".into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }
        Err(_) => {
            backend.busy.store(false, Ordering::SeqCst);
            return Err("Installation state is unavailable".into());
        }
    }
    let worker = backend.inner().clone();
    let spawned = std::thread::Builder::new()
        .name("java-install".into())
        .spawn(move || {
            struct Release(Arc<Backend>);
            impl Drop for Release {
                fn drop(&mut self) {
                    self.0.busy.store(false, Ordering::SeqCst);
                }
            }
            let _release = Release(worker);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                launcher_install::java::install_managed_java(&root, major, &cancel, &|p| {
                    if let Ok(mut t) = task.lock() {
                        t.progress = p;
                    }
                })
            }))
            .unwrap_or_else(|_| Err("Java installation stopped unexpectedly".into()));
            if let Ok(mut t) = task.lock() {
                t.running = false;
                t.cancelling = false;
                match result {
                    Ok(java) => {
                        t.result = Some(java);
                        t.progress.phase = "Installation complete".into();
                    }
                    Err(e) => t.error = Some(e),
                }
            }
        });
    if let Err(e) = spawned {
        backend.busy.store(false, Ordering::SeqCst);
        if let Ok(mut t) = installer.task.lock() {
            t.running = false;
            t.error = Some(e.to_string());
        }
        return Err(e.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn create_instance(
    backend: tauri::State<'_, Arc<Backend>>, installer: tauri::State<'_, Installer>,
    item: launcher_core::Instance, revision: u64,
) -> Result<launcher_core::Library, String> {
    launcher_install::require_platform()?;
    let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    backend.ensure_idle()?;
    let storage = guard.as_ref().map_err(Clone::clone)?;
    let library = crate::configuration::create(&storage.store(), &storage.location.root, item, revision)?;
    let item = library.instances.last().ok_or("Failed to create the instance")?.clone();
    // The record and fresh directory now exist. Failed worker startup remains retryable.
    if let Err(error) = start_worker(backend.inner().clone(), &installer, storage.location.root.clone(), item) {
        if let Ok(mut task) = installer.task.lock() { task.running = false; task.error = Some(error); }
    }
    Ok(library)
}
#[tauri::command]
pub async fn delete_instance(
    backend: tauri::State<'_, Arc<Backend>>, instance_id: String, revision: u64,
) -> Result<launcher_core::Library, String> {
    let backend = backend.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
        backend.ensure_idle()?;
        let storage = guard.as_ref().map_err(Clone::clone)?;
        crate::configuration::delete(&storage.store(), &storage.location.root, &instance_id, revision)
    }).await.map_err(|e| e.to_string())?
}


#[tauri::command]
pub async fn list_loader_versions(kind: String, game: String) -> Result<Vec<launcher_install::loaders::LoaderVersion>, String> {
    tauri::async_runtime::spawn_blocking(move || launcher_install::loaders::catalog(&kind, &game)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn validate_installation_java(version: String, path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let major = minecraft::recommendation(&version)?;
        launcher_install::java::validate_selected(std::path::Path::new(&path), major, &AtomicBool::new(false))?;
        Ok(())
    }).await.map_err(|e| e.to_string())?
}
