#[tauri::command]
pub fn available_memory_bytes()->Result<u64,String> {launcher_platform::available_memory_bytes()}
