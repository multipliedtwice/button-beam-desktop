use crate::shortcuts::{simulate_shortcut, ShortcutStore};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::Mutex;
use warp::filters::ws::WebSocket;
use warp::ws::Message;
use warp::Filter; // Import Manager trait

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub connected: bool,
}

#[derive(Debug)]
pub struct AppState {
    pub devices: Mutex<HashMap<String, Device>>, // Asynchronous Mutex for thread-safe access
}

impl AppState {
    /// Creates a new AppState with an empty devices HashMap.
    pub fn new() -> Self {
        Self {
            devices: Mutex::new(HashMap::new()),
        }
    }

    /// Retrieves a list of all connected devices.
    pub async fn get_connected_devices(&self) -> Vec<Device> {
        let devices = self.devices.lock().await;
        devices.values().cloned().collect()
    }
}

/// Starts the WebSocket server.
///
/// # Arguments
///
/// * `ip` - The IP address to bind the server to.
/// * `port` - The port number to bind the server to.
/// * `store` - Shared state containing shortcuts.
/// * `app_state` - Shared state containing connected devices.
/// * `app_handle` - Handle to emit events to the frontend.
pub async fn start_websocket_server(
    ip: String,
    port: u16,
    store: Arc<ShortcutStore>,
    app_state: Arc<AppState>,
    app_handle: AppHandle,
) {
    let store_filter = warp::any().map(move || store.clone());
    let app_state_filter = warp::any().map(move || app_state.clone());
    let app_handle_filter = warp::any().map(move || app_handle.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(store_filter)
        .and(app_state_filter)
        .and(app_handle_filter)
        .map(
            |ws: warp::ws::Ws,
             store: Arc<ShortcutStore>,
             app_state: Arc<AppState>,
             app_handle: AppHandle| {
                ws.on_upgrade(move |websocket| {
                    handle_websocket_connection(
                        websocket,
                        store.clone(),
                        app_state.clone(),
                        app_handle.clone(),
                    )
                })
            },
        );

    let addr = format!("{}:{}", ip, port);
    println!("WebSocket server listening on ws://{}", addr);

    warp::serve(ws_route)
        .run(addr.parse::<std::net::SocketAddr>().unwrap())
        .await;
}

/// Handles individual WebSocket connections.
///
/// # Arguments
///
/// * `websocket` - The WebSocket connection.
/// * `store` - Shared state containing shortcuts.
/// * `app_state` - Shared state containing connected devices.
/// * `app_handle` - Handle to emit events to the frontend.
pub async fn handle_websocket_connection(
    websocket: WebSocket,
    store: Arc<ShortcutStore>,
    app_state: Arc<AppState>,
    app_handle: AppHandle,
) {
    let (ws_sender, mut ws_receiver) = websocket.split();

    // Wrap ws_sender in Arc<Mutex<>> for shared access
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    // Subscribe to the broadcaster for shortcut updates
    let mut receiver = store.broadcaster.subscribe();

    // Shared state for device name
    let device_name = Arc::new(Mutex::new(None));

    // Clone Arcs for the send task
    let send_ws_sender = Arc::clone(&ws_sender);

    // Task to send shortcut updates to the client
    let send_task = tokio::spawn(async move {
        while let Ok(shortcut) = receiver.recv().await {
            let shortcut_json = serde_json::to_string(&shortcut).unwrap();
            let mut sender_guard = send_ws_sender.lock().await;
            if sender_guard
                .send(Message::text(shortcut_json))
                .await
                .is_err()
            {
                // Client disconnected
                break;
            }
        }
    });

    // Clone Arcs for the receive task
    let recv_store = Arc::clone(&store);
    let recv_app_state = Arc::clone(&app_state);
    let recv_app_handle = app_handle.clone();
    let recv_device_name = Arc::clone(&device_name);
    let recv_ws_sender = Arc::clone(&ws_sender);

    // Task to receive messages from the client
    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(message) => {
                    if let Ok(text) = message.to_str() {
                        if let Ok(data) = serde_json::from_str::<serde_json::Value>(text) {
                            match data.get("type").and_then(|t| t.as_str()) {
                                Some("device_info") => {
                                    if let Some(name) =
                                        data.get("device_name").and_then(|n| n.as_str())
                                    {
                                        // Update device_name
                                        {
                                            let mut device_lock = recv_device_name.lock().await;
                                            *device_lock = Some(name.to_string());
                                        }

                                        // Add device to AppState
                                        {
                                            let mut devices = recv_app_state.devices.lock().await;
                                            devices.insert(
                                                name.to_string(),
                                                Device {
                                                    name: name.to_string(),
                                                    connected: true,
                                                },
                                            );
                                        }

                                        println!("Device connected: {}", name);

                                        // Emit an event to notify frontend about the new device
                                        let connected_devices =
                                            recv_app_state.get_connected_devices().await;
                                        recv_app_handle
                                            .emit_all("devices_updated", connected_devices)
                                            .unwrap();

                                        // Send the current list of shortcuts to the newly connected device
                                        let all_shortcuts = recv_store.get_shortcuts();
                                        let shortcuts_json =
                                            serde_json::to_string(&all_shortcuts).unwrap();
                                        let mut sender_guard = recv_ws_sender.lock().await;
                                        if sender_guard
                                            .send(Message::text(shortcuts_json))
                                            .await
                                            .is_err()
                                        {
                                            // Client disconnected
                                            break;
                                        }
                                    }
                                }
                                Some("execute_shortcut") => {
                                    if let Some(shortcut_id) =
                                        data.get("shortcut_id").and_then(|id| id.as_u64())
                                    {
                                        if let Some(shortcut) = recv_store
                                            .shortcuts
                                            .lock()
                                            .unwrap()
                                            .iter()
                                            .find(|s| s.id == shortcut_id)
                                        {
                                            if let Err(e) = simulate_shortcut(shortcut.keys.clone())
                                            {
                                                eprintln!("Error simulating shortcut: {}", e);
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    println!("Unknown message type or missing type field");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        // Handle device disconnection
        if let Some(name) = device_name.lock().await.clone() {
            let mut devices = recv_app_state.devices.lock().await;
            devices.remove(&name);
            println!("Device disconnected: {}", name);

            // Emit an event to notify frontend about the device disconnection
            let connected_devices = recv_app_state.get_connected_devices().await;
            recv_app_handle
                .emit_all("devices_updated", connected_devices)
                .unwrap();
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}
