//! IPC and host orchestration only; authentication policy stays in microsoft_auth.
use crate::{accounts::AccountStore, microsoft_auth::{self, MicrosoftLogin, LoginStatus}};
#[tauri::command]
pub fn start_microsoft_login(session:tauri::State<'_,MicrosoftLogin>,store:tauri::State<'_,AccountStore>)->Result<(),String> {
    microsoft_auth::start_microsoft_login(&session,&store)
}
#[tauri::command]
pub fn microsoft_login_status(session:tauri::State<'_,MicrosoftLogin>)->Result<LoginStatus,String> {
    microsoft_auth::microsoft_login_status(&session)
}
#[tauri::command]
pub fn cancel_microsoft_login(session:tauri::State<'_,MicrosoftLogin>)->Result<(),String> {
    microsoft_auth::cancel_microsoft_login(&session)
}
#[tauri::command]
pub fn open_microsoft_login(session:tauri::State<'_,MicrosoftLogin>)->Result<(),String> {
    let uri=microsoft_auth::verification_address(&session)?;
    launcher_platform::open_external(&uri)
        .map_err(|_|"Cannot open browser; open the displayed address manually".into())
}
