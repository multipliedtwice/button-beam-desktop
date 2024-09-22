mod shortcuts;
mod sockets;

use crate::shortcuts::{
    add_shortcut, delete_shortcut, get_shortcuts_command, simulate_shortcut, update_shortcut,
    ShortcutStore,
};
use crate::sockets::{start_websocket_server, AppState};
use shortcuts::Shortcut;
use std::net::{Ipv4Addr, TcpListener};
use std::sync::Arc;
use tokio::sync::broadcast;

#[tauri::command]
fn get_local_ip() -> Result<String, String> {
    match local_ipaddress::get() {
        Some(ip) => Ok(ip),
        None => Err("Unable to get local IP address".into()),
    }
}

#[tauri::command]
fn find_free_port() -> Result<u16, String> {
    // Bind to address 0.0.0.0 with port 0 to let the OS assign a free port
    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, 0))
        .map_err(|e| format!("Failed to bind to a free port: {}", e))?;
    let port = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?
        .port();
    Ok(port)
}

fn main() {
    // Generate the Tauri context
    let context = tauri::generate_context!();

    // Determine the application data directory
    let app_dir = tauri::api::path::app_data_dir(&context.config())
        .expect("Cannot locate app data directory");
    let shortcuts_file = app_dir.join("shortcuts.json");

    // Create a broadcast channel for shortcut updates
    let (sender, _receiver) = broadcast::channel::<Shortcut>(16);

    // Initialize the ShortcutStore with broadcaster
    let store = Arc::new(ShortcutStore::new(shortcuts_file, sender.clone()));

    // Initialize the AppState with an empty HashMap for devices
    let app_state = Arc::new(AppState::new());

    // Get the local IP address and find a free port
    let ip = get_local_ip().unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = find_free_port().unwrap_or(3000);

    tauri::Builder::default()
        .manage(store.clone())
        .manage(app_state.clone())
        .invoke_handler(tauri::generate_handler![
            get_shortcuts_command,
            add_shortcut,
            update_shortcut,
            delete_shortcut,
            simulate_shortcut,
            get_local_ip,
            find_free_port,
        ])
        .setup(move |app| {
            let handle = app.handle();
            tauri::async_runtime::spawn(start_websocket_server(
                ip.clone(),
                port,
                store.clone(),
                app_state.clone(),
                handle.clone(),
            ));
            println!("WebSocket server started at ws://{}:{}", ip, port);
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
