//! IPC and host orchestration only; authentication policy stays in microsoft_auth.
//! The sign-in window is a plain Microsoft page: it has no IPC capability, and its
//! only interaction with the launcher is the intercepted official redirect.
use crate::{accounts::AccountStore, microsoft_auth::{self, MicrosoftLogin, LoginStatus}};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
const LOGIN_WINDOW: &str = "microsoft-login";
#[tauri::command]
pub fn start_microsoft_login(app:AppHandle,session:tauri::State<'_,MicrosoftLogin>,store:tauri::State<'_,AccountStore>)->Result<(),String> {
    close_login_window(&app);
    microsoft_auth::start_microsoft_login(&session,&store,move |id,uri| {
        if show_login_window(&app,id,uri).is_err() { microsoft_auth::abandon_authorization(&app.state::<MicrosoftLogin>(),id); }
    })
}
#[tauri::command]
pub fn microsoft_login_status(session:tauri::State<'_,MicrosoftLogin>)->Result<LoginStatus,String> {
    microsoft_auth::microsoft_login_status(&session)
}
#[tauri::command]
pub fn cancel_microsoft_login(app:AppHandle,session:tauri::State<'_,MicrosoftLogin>)->Result<(),String> {
    microsoft_auth::cancel_microsoft_login(&session)?;
    close_login_window(&app); Ok(())
}
/// Brings the sign-in window back (or reopens it) for the attempt still waiting.
#[tauri::command]
pub fn open_microsoft_login(app:AppHandle,session:tauri::State<'_,MicrosoftLogin>)->Result<(),String> {
    let (id,uri)=microsoft_auth::authorization_address(&session)?;
    show_login_window(&app,id,&uri)
}
fn close_login_window(app:&AppHandle) {
    let handle=app.clone();
    let _=app.run_on_main_thread(move || { if let Some(window)=handle.get_webview_window(LOGIN_WINDOW) { let _=window.destroy(); } });
}
fn show_login_window(app:&AppHandle,id:u64,uri:&str)->Result<(),String> {
    const OPEN:&str="Cannot open the Microsoft sign-in window";
    let url=tauri::Url::parse(uri).map_err(|_|OPEN)?;
    if let Some(window)=app.get_webview_window(LOGIN_WINDOW) {
        let _=window.unminimize(); let _=window.show(); window.set_focus().map_err(|_|OPEN)?; return Ok(());
    }
    let (navigation,closed)=(app.clone(),app.clone());
    let window=WebviewWindowBuilder::new(app,LOGIN_WINDOW,WebviewUrl::External(url))
        .title("Microsoft").inner_size(520.0,720.0).min_inner_size(400.0,560.0).center().incognito(true)
        .on_navigation(move |url| {
            if !microsoft_auth::is_redirect(url.as_str()) { return url.scheme()=="https"; }
            let _=microsoft_auth::deliver_redirect(&navigation.state::<MicrosoftLogin>(),url.as_str());
            close_login_window(&navigation);
            false
        })
        .build().map_err(|_|OPEN)?;
    window.on_window_event(move |event| if let WindowEvent::Destroyed=event {
        microsoft_auth::abandon_authorization(&closed.state::<MicrosoftLogin>(),id);
    });
    Ok(())
}
