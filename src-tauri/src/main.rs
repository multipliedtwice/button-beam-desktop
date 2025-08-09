/// ./src-tauri/src/main.rs
mod shortcuts;
mod sockets;

use crate::shortcuts::{
    add_shortcut, delete_shortcut, get_shortcuts_command, register_global_shortcuts,
    simulate_shortcut, simulate_shortcut_by_id, update_shortcut, Shortcut, ShortcutStore,
};

use crate::sockets::{start_websocket_server, AppState};
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::broadcast;

struct ServerConfig {
    ip: String,
    port: u16,
}

#[derive(serde::Serialize)]
struct ServerConfigData {
    ip: String,
    port: u16,
}

#[tauri::command]
fn get_local_ip() -> Result<String, String> {
    match local_ipaddress::get() {
        Some(ip) => Ok(ip),
        None => Err("Unable to get local IP address".into()),
    }
}

#[tauri::command]
fn find_free_port() -> Result<u16, String> {
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))
        .map_err(|e| format!("Failed to bind to a free port: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    Ok(port)
}

#[tauri::command]
fn get_server_config(server_config: State<Arc<ServerConfig>>) -> ServerConfigData {
    ServerConfigData {
        ip: server_config.ip.clone(),
        port: server_config.port,
    }
}

fn main() {
    let context = tauri::generate_context!();

    let app_dir = tauri::api::path::app_data_dir(&context.config())
        .expect("Cannot locate app data directory");
    let shortcuts_file = app_dir.join("shortcuts.json");

    let (sender, _receiver) = broadcast::channel::<Vec<Shortcut>>(16);

    let store = Arc::new(ShortcutStore::new(shortcuts_file, sender.clone()));
    let app_state = Arc::new(AppState::new());

    let store_clone = Arc::clone(&store); // Clone store here
    let app_state_clone = Arc::clone(&app_state); // Clone app_state here

    tauri::Builder::default()
        .setup(move |app| {
            let ip = get_local_ip().unwrap_or_else(|_| "127.0.0.1".to_string());
            let port = find_free_port().unwrap_or(3000);

            let server_config = Arc::new(ServerConfig {
                ip: ip.clone(),
                port,
            });
            app.manage(server_config);

            let app_handle = app.handle();
            let ip_clone = ip.clone();

            // Clone variables before moving into the closure
            let store_clone_for_ws = Arc::clone(&store_clone);
            let app_handle_for_ws = app_handle.clone();
            let app_state_clone_for_ws = Arc::clone(&app_state_clone);

            tauri::async_runtime::spawn(async move {
                start_websocket_server(
                    &ip,
                    port,
                    store_clone_for_ws,
                    app_state_clone_for_ws,
                    app_handle_for_ws,
                )
                .await;
            });

            println!("WebSocket server started at ws://{}:{}", ip_clone, port);

            // Register global shortcuts
            let store_clone_for_shortcuts = Arc::clone(&store_clone);
            let app_handle_for_shortcuts = app_handle.clone();
            register_global_shortcuts(app_handle_for_shortcuts, store_clone_for_shortcuts);

            Ok(())
        })
        .manage(Arc::clone(&store)) // Use cloned `store` here
        .manage(Arc::clone(&app_state)) // Use cloned `app_state` here
        .invoke_handler(tauri::generate_handler![
            get_shortcuts_command,
            add_shortcut,
            update_shortcut,
            delete_shortcut,
            simulate_shortcut,
            simulate_shortcut_by_id,
            get_local_ip,
            get_server_config,
        ])
        .run(context)
        .expect("error while running tauri application");
}
