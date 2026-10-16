use crate::storage_commands::Backend;
use serde::{Deserialize, Serialize};
use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}, fs};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences { #[serde(default)] pub steam_adaptation: bool }
pub struct SteamSession { pub launched: bool, pub active: AtomicBool }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status { enabled: bool, launched: bool, active: bool }
fn steam_id(value: &str) -> bool { !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) && value.bytes().any(|b| b != b'0') }
pub fn detected() -> bool {
    std::env::args().any(|arg| arg == "--steam-entry") || ["SteamGameId", "SteamAppId"].iter().any(|key| std::env::var(key).is_ok_and(|v| steam_id(&v)))
}
pub fn read(backend: &Backend) -> Result<Preferences, String> {
    let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    let path = guard.as_ref().map_err(Clone::clone)?.launcher_root.join("launcher-settings.json");
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|e| format!("Launcher settings are corrupt: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(e) => Err(e.to_string()),
    }
}
#[tauri::command]
pub fn steam_status(backend: tauri::State<'_, Arc<Backend>>, session: tauri::State<'_, SteamSession>) -> Result<Status, String> {
    backend.ensure_idle()?;
    Ok(Status { enabled: read(&backend)?.steam_adaptation, launched: session.launched, active: session.active.load(Ordering::SeqCst) })
}
#[tauri::command]
pub fn set_steam_adaptation(enabled: bool, window: tauri::WebviewWindow, backend: tauri::State<'_, Arc<Backend>>, session: tauri::State<'_, SteamSession>) -> Result<Status, String> {
    backend.ensure_idle()?;
    let previous = read(&backend)?.steam_adaptation;
    let active = enabled && session.launched;
    if session.launched { window.set_fullscreen(active).map_err(|e| e.to_string())?; }
    let saved = (|| {
        let guard = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
        backend.ensure_idle()?;
        launcher_core::storage::atomic_write(&guard.as_ref().map_err(Clone::clone)?.launcher_root.join("launcher-settings.json"), &serde_json::to_vec(&Preferences { steam_adaptation: enabled }).map_err(|e| e.to_string())?)
    })();
    if let Err(e) = saved { if session.launched { let _ = window.set_fullscreen(previous); } return Err(e); }
    session.active.store(active, Ordering::SeqCst);
    Ok(Status { enabled, launched: session.launched, active })
}
#[tauri::command]
pub fn request_close(window: tauri::WebviewWindow, backend: tauri::State<'_, Arc<Backend>>, game: tauri::State<'_, crate::launch_commands::GameLauncher>) -> Result<(), String> {
    if game.defer_close(&window)? { return Ok(()); }
    backend.ensure_idle()?;
    window.close().map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn steam_keyboard(show: bool, session: tauri::State<'_, SteamSession>) -> Result<(), String> {
    if show && !session.active.load(Ordering::SeqCst) { return Err("Steam adaptation is not enabled".into()); }
    if !session.launched { return Ok(()); }
    tauri::async_runtime::spawn_blocking(move || {
        launcher_platform::steam_keyboard(show)
    }).await.map_err(|e| e.to_string())?
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn ids_and_old_settings() {
        for v in ["", "0", "000", "abc", "-1", " 42"] { assert!(!steam_id(v)); }
        for v in ["42", "18446744073709551615"] { assert!(steam_id(v)); }
        assert!(!serde_json::from_str::<Preferences>("{}").unwrap().steam_adaptation);
    }
}
