//! Window focus and IPC orchestration only; acquisition lives in launcher-platform.
use launcher_platform::gamepad::Batch;
pub use launcher_platform::gamepad::NativeGamepads;

#[tauri::command]
pub fn poll_gamepads(
    window: tauri::WebviewWindow,
    gamepads: tauri::State<'_, NativeGamepads>,
) -> Result<Batch, String> {
    let suspended = !window.is_focused().map_err(|e| e.to_string())?;
    gamepads.poll(suspended)
}
