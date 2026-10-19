//! Bounded clipboard IPC; toolkit work is dispatched to the host main thread.
#[tauri::command]
pub async fn copy_clipboard_text(app:tauri::AppHandle,text:String)->Result<bool,String> {
    if text.len()>8*1024*1024 || text.contains('\0') {return Err("Clipboard is unavailable".into());}
    let (sender,receiver)=std::sync::mpsc::sync_channel(1);
    app.run_on_main_thread(move || {let _=sender.send(crate::host::copy_clipboard_text(&text));})
        .map_err(|_|"Clipboard is unavailable")?;
    tauri::async_runtime::spawn_blocking(move || receiver.recv().map_err(|_|"Clipboard is unavailable".to_string()))
        .await.map_err(|_|"Clipboard is unavailable")?
}
