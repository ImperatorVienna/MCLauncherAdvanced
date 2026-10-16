use launcher_core::storage::Storage;
use std::{fs, sync::{Mutex, atomic::{AtomicBool, Ordering}}};
use tauri::Manager;

pub struct Backend {
    pub storage: Mutex<Result<Storage, String>>,
    pub busy: AtomicBool,
    // One session per portable launcher directory.
    _session: fs::File,
}
impl Backend {
    pub fn initialize(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let bootstrap = launcher_directory()?;
        fs::create_dir_all(&bootstrap)?;
        let session = fs::OpenOptions::new().create(true).truncate(false).write(true).open(bootstrap.join("session.lock"))?;
        session.try_lock().map_err(|_| "The launcher is already running; use the existing window")?;
        let storage = crate::loader_management::recover(&bootstrap.join(".minecraft")).and_then(|_| Storage::open(&bootstrap))
            .and_then(|storage| {
                launcher_platform::register(&bootstrap, &app.path().data_dir().map_err(|e|e.to_string())?, include_bytes!("../icons/icon.png"))?;
                crate::local_instances::discover(&storage)?;
                crate::java_selection::reconcile(&storage)?;
                Ok(storage)
            });
        Ok(Self { storage: Mutex::new(storage), busy: AtomicBool::new(false), _session: session })
    }
    pub fn ensure_idle(&self) -> Result<(), String> {
        if self.busy.load(Ordering::SeqCst) { Err("A file task is running; wait for completion".into()) } else { Ok(()) }
    }
    #[cfg(test)]
    pub fn for_test(path: std::path::PathBuf) -> Self {
        fs::create_dir_all(&path).unwrap();
        let session = fs::File::create(path.join("session.lock")).unwrap();
        Self { storage: Mutex::new(Storage::open(&path)), busy: AtomicBool::new(false), _session: session }
    }
}
pub use launcher_platform::launcher_directory;
