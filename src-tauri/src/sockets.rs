use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use warp::filters::ws::WebSocket;
use warp::ws::Message;
use warp::Filter;

use crate::shortcuts::{simulate_shortcut, ShortcutStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub name: String,
    pub connected: bool,
}

pub struct AppState {
    pub device: Mutex<Option<Device>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            device: Mutex::new(None),
        }
    }
}

pub async fn start_websocket_server(
    ip: &str,
    port: u16,
    store: Arc<ShortcutStore>,
    app_state: Arc<AppState>,
    app_handle: tauri::AppHandle,
) {
    let ws_route = warp::path::end()
        .and(warp::ws())
        .and(warp::any().map(move || store.clone()))
        .and(warp::any().map(move || app_state.clone()))
        .and(warp::any().map(move || app_handle.clone()))
        .map(
            |ws: warp::ws::Ws,
             store: Arc<ShortcutStore>,
             app_state: Arc<AppState>,
             app_handle: tauri::AppHandle| {
                ws.on_upgrade(move |websocket| {
                    handle_websocket_connection(websocket, store, app_state, app_handle)
                })
            },
        );

    let addr = format!("{}:{}", ip, port);
    println!("WebSocket server listening on ws://{}", addr);

    warp::serve(ws_route)
        .run(addr.parse::<std::net::SocketAddr>().unwrap())
        .await;
}

pub async fn handle_websocket_connection(
    websocket: WebSocket,
    store: Arc<ShortcutStore>,
    app_state: Arc<AppState>,
    app_handle: tauri::AppHandle,
) {
    let (ws_sender, mut ws_receiver) = websocket.split();
    let send_ws_sender = Arc::new(Mutex::new(ws_sender));

    {
        let device_lock = app_state.device.lock().await;
        if device_lock.is_some() {
            println!("A device is already connected. Rejecting new connection.");
            let rejection_message = Message::text("connection_rejected");
            send_ws_sender
                .lock()
                .await
                .send(rejection_message)
                .await
                .ok();
            return;
        } else {
            println!("New connection attempt.");
        }
    }

    let app_handle_clone = app_handle.clone();
    let recv_store = Arc::clone(&store);
    let recv_app_state = Arc::clone(&app_state);
    let send_ws_sender_clone = Arc::clone(&send_ws_sender);

    let recv_task = tokio::spawn(async move {
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(message) => {
                    if let Ok(text) = message.to_str() {
                        if let Ok(data) = serde_json::from_str::<Value>(text) {
                            match data.get("type").and_then(|t| t.as_str()) {
                                Some("device_info") => {
                                    handle_device_info(
                                        data,
                                        recv_app_state.clone(),
                                        app_handle_clone.clone(),
                                        send_ws_sender_clone.clone(),
                                        recv_store.clone(),
                                    )
                                    .await;
                                }
                                Some("execute_shortcut") => {
                                    handle_execute_shortcut(data, recv_store.clone()).await;
                                }
                                _ => println!("Unknown message type or missing type field."),
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

        if recv_app_state.device.lock().await.is_some() {
            println!("Device disconnected.");
            let mut device_lock = recv_app_state.device.lock().await;
            *device_lock = None;

            // Emit events on device disconnection
            app_handle_clone
                .emit_all("devices_updated", None::<&Device>)
                .unwrap();
        }
    });

    tokio::select! {
        _ = recv_task => {},
    }
}

async fn handle_device_info(
    data: Value,
    app_state: Arc<AppState>,
    app_handle: tauri::AppHandle,
    send_ws_sender: Arc<Mutex<futures_util::stream::SplitSink<WebSocket, Message>>>,
    store: Arc<ShortcutStore>,
) {
    if let Some(name) = data.get("device_name").and_then(|n| n.as_str()) {
        println!("Device connected: {}", name);
        let mut device_lock = app_state.device.lock().await;

        *device_lock = Some(Device {
            name: name.to_string(),
            connected: true,
        });

        // Emit events
        app_handle
            .emit_all("devices_updated", &*device_lock)
            .unwrap();
        app_handle
            .emit_all("device_connected", &*device_lock)
            .unwrap();

        // Send shortcuts to client
        let all_shortcuts = store.get_shortcuts();
        let shortcuts_json = serde_json::to_string(&all_shortcuts).unwrap();
        let mut sender_guard = send_ws_sender.lock().await;

        sender_guard.send(Message::text(shortcuts_json)).await.ok();
    }
}

async fn handle_execute_shortcut(data: Value, store: Arc<ShortcutStore>) {
    if let Some(shortcut_id) = data.get("shortcut_id").and_then(|id| id.as_i64()) {
        println!("Executing shortcut with ID: {}", shortcut_id);

        let all_shortcuts = store.get_shortcuts();

        // Find the shortcut by ID
        if let Some(shortcut) = all_shortcuts.iter().find(|s| s.id == shortcut_id as u64) {
            println!("Found shortcut: {:?}", shortcut);

            // Here we assume there's a field `interval_ms` in the incoming data
            let interval_ms = data.get("interval_ms").and_then(|i| i.as_u64());

            // Use the simulate_shortcut function to simulate the key presses
            if let Err(e) = simulate_shortcut(shortcut.sequence.clone(), interval_ms) {
                eprintln!("Failed to simulate shortcut: {}", e);
            }
        } else {
            eprintln!("Shortcut with ID {} not found.", shortcut_id);
        }
    }
}
