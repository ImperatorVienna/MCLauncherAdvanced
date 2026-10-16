#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod updater;
mod content_commands;
mod content_downloads;
mod memory;
mod java_selection;
mod local_instances;
mod logs;
mod loader_management;
mod accounts;
mod microsoft_auth;
mod skins;
mod launch_plan;
mod launch_commands;
mod configuration;
mod gamepad;
mod install_commands;
mod steam;
mod storage_commands;
use launcher_core::Library;
#[cfg(test)]
use launcher_core::Store;
use tauri::Manager;
// Kept as thin commands; navigation never enters the backend.
#[tauri::command]
fn load_library(
    backend: tauri::State<'_, std::sync::Arc<storage_commands::Backend>>,
) -> Result<Library, String> {
    let storage = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    storage.as_ref().map_err(Clone::clone)?.store().load().inspect_err(|e|logs::record(&format!("Loading instance library failed: {e}")))
}
#[tauri::command]
fn save_library(
    backend: tauri::State<'_, std::sync::Arc<storage_commands::Backend>>,
    library: Library,
) -> Result<Library, String> {
    let storage = backend.storage.lock().map_err(|_| "Instance storage is unavailable")?;
    backend.ensure_idle()?;
    configuration::save_metadata(&storage.as_ref().map_err(Clone::clone)?.store(), library).inspect_err(|e|logs::record(&format!("Saving instance library failed: {e}")))
}
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let _ = logs::initialize(&storage_commands::launcher_directory()?);
            let backend = storage_commands::Backend::initialize(app.handle()).map_err(|e| {logs::record("Launcher storage initialization failed"); e})?;
            app.manage(accounts::initialize(app.handle())?);
            app.manage(microsoft_auth::MicrosoftLogin::default());
            app.manage(skins::initialize(app.handle())?);
            let launched = steam::detected();
            let enabled = steam::read(&backend)
                .map(|p| p.steam_adaptation)
                .unwrap_or(false);
            let active = enabled && launched;
            app.manage(steam::SteamSession {
                launched,
                active: std::sync::atomic::AtomicBool::new(active),
            });
            app.manage(std::sync::Arc::new(backend));
            app.manage(gamepad::NativeGamepads::start());
            app.manage(launch_commands::GameLauncher::default());
            app.manage(install_commands::Installer::default());
            let updater = std::sync::Arc::new(updater::Updater::default());
            let _ = updater.check(app.package_info().version.to_string());
            app.manage(updater);
            // Create the first WebView only after overriding both WebKit data/cache roots.
            let webview_data = storage_commands::launcher_directory()?.join("webview");
            std::fs::create_dir_all(&webview_data)?;
            tauri::WebviewWindowBuilder::from_config(app, &app.config().app.windows[0])?
                .icon(app.default_window_icon().cloned().ok_or("Launcher icon is missing")?)?
                .data_directory(webview_data)
                .fullscreen(active)
                .build()?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                logs::record("Window close requested");
                if let Some(webview) = window.get_webview_window(window.label()) {
                    if window.state::<launch_commands::GameLauncher>().defer_close(&webview).unwrap_or(false) {
                        api.prevent_close();
                        return;
                    }
                }
                if window
                    .state::<std::sync::Arc<storage_commands::Backend>>()
                    .busy
                    .load(std::sync::atomic::Ordering::SeqCst)
                {
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            updater::launcher_update_status, updater::check_launcher_update, updater::download_launcher_update, updater::cancel_launcher_update, updater::apply_launcher_update,
            content_downloads::identify_instance_content,
            content_downloads::search_instance_content,
            content_downloads::check_content_updates,
            content_downloads::catalog_content_states,
            content_downloads::validate_content_updates,
            content_downloads::start_content_installation,
            content_commands::delete_instance_content,
            content_commands::scan_instance_content,
            content_commands::set_instance_content_enabled,
            content_commands::open_instance_content_directory,
            logs::list_logs, logs::read_log, logs::open_log_location,
            microsoft_auth::start_microsoft_login, microsoft_auth::microsoft_login_status, microsoft_auth::cancel_microsoft_login, microsoft_auth::open_microsoft_login,
            accounts::load_accounts, accounts::save_accounts,
            skins::load_skins, skins::save_skins,
            launch_commands::launch_game, launch_commands::game_status,
            memory::available_memory_bytes,
            load_library,
            save_library,
            install_commands::create_instance,
            install_commands::delete_instance,
            steam::steam_status,
            steam::set_steam_adaptation,
            steam::steam_keyboard,
            steam::request_close,
            gamepad::poll_gamepads,
            install_commands::list_java,
            install_commands::recommended_java,
            install_commands::validate_installation_java,
            install_commands::start_java_installation,
            install_commands::install_platform,
            install_commands::list_minecraft_versions, install_commands::list_loader_versions,
            loader_management::change_mod_loader,
            install_commands::installation_task,
            install_commands::start_installation,
            install_commands::cancel_installation,
            install_commands::installed_versions
        ])
        .run(tauri::generate_context!())
        .expect("failed to run launcher desktop shell");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ipc_load_save_fixed_root_and_reject_stale_write() {
        let path = std::env::temp_dir().join(format!(
            "launcher-ipc-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let app = tauri::test::mock_builder()
            .manage(std::sync::Arc::new(storage_commands::Backend::for_test(
                path.clone(),
            )))
            .manage(install_commands::Installer::default())
            .invoke_handler(tauri::generate_handler![
            memory::available_memory_bytes,
                load_library,
                save_library,
            install_commands::create_instance,
            install_commands::delete_instance,
                install_commands::installation_task,
                install_commands::cancel_installation,
                install_commands::start_installation
            ])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let webview = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let invoke = |cmd: &str, body: serde_json::Value| {
            tauri::test::get_ipc_response(
                &webview,
                tauri::webview::InvokeRequest {
                    cmd: cmd.into(),
                    callback: tauri::ipc::CallbackFn(0),
                    error: tauri::ipc::CallbackFn(1),
                    url: "tauri://localhost".parse().unwrap(),
                    body: tauri::ipc::InvokeBody::Json(body),
                    headers: Default::default(),
                    invoke_key: tauri::test::INVOKE_KEY.into(),
                },
            )
        };
        assert!(invoke(
            "migrate_root",
            serde_json::json!({ "target": path.join("other") })
        )
        .is_err());
        let initial = invoke("load_library", serde_json::json!({}))
            .unwrap()
            .deserialize::<Library>()
            .unwrap();
        assert_eq!(initial.revision, 0);
        let mut data = initial;
        data.instances.push(launcher_core::Instance {
            id: "ipc-test".into(),
            name: "IPC instance".into(),
            minecraft_version: "1.21.1".into(),
            icon: "sand".into(),
            java_path: None, loader: None, launch_settings: Default::default(),
            directory_name: "1.21.1".into(),
        });
        data.selected_id = Some("ipc-test".into());
        let saved = Store::new(path.join("launcher-library.json")).save(data.clone()).unwrap();
        assert_eq!(saved.revision, 1);
        assert!(invoke("save_library", serde_json::json!({ "library": data })).is_err());
        let loaded = Store::new(path.join("launcher-library.json"))
            .load()
            .unwrap();
        assert_eq!(loaded.instances[0].name, "IPC instance");
        let after = invoke("load_library", serde_json::json!({}))
            .unwrap()
            .deserialize::<Library>()
            .unwrap();
        assert_eq!(after.revision, 1);
        assert_eq!(after.instances[0].name, "IPC instance");
        let task = invoke("installation_task", serde_json::json!({}))
            .unwrap()
            .deserialize::<serde_json::Value>()
            .unwrap();
        assert_eq!(task["running"], false);
        assert!(invoke(
            "start_installation",
            serde_json::json!({"instanceId":"missing"})
        )
        .is_err());
        app.state::<std::sync::Arc<storage_commands::Backend>>()
            .busy
            .store(true, std::sync::atomic::Ordering::SeqCst);
        assert!(invoke(
            "start_installation",
            serde_json::json!({"instanceId":"ipc-test"})
        )
        .is_err());
        assert!(invoke("save_library", serde_json::json!({"library":after})).is_err());
        assert!(invoke("load_library", serde_json::json!({})).is_ok());
        assert!(invoke("cancel_installation", serde_json::json!({})).is_ok());
        std::fs::remove_dir_all(path).unwrap();
    }
}
