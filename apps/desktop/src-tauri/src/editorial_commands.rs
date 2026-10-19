//! IPC validation and host dispatch; content policy lives in the service.
#[tauri::command]
pub async fn official_content(path: String) -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(move || launcher_install::editorial::fetch(&path)).await.map_err(|e|e.to_string())?
}
#[tauri::command]
pub fn open_information_link(url: String) -> Result<(), String> {
    if !launcher_install::editorial::allowed_link(&url) {return Err("Unsupported information link".into());}
    launcher_platform::open_external(url)
}
