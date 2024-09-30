use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};
use tokio::sync::broadcast::Sender;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Shortcut {
    pub id: u64,
    pub name: String,
    pub sequence: Vec<String>,
}

pub struct ShortcutStore {
    pub shortcuts: Mutex<Vec<Shortcut>>,
    pub file_path: PathBuf,
    pub broadcaster: Sender<Vec<Shortcut>>,
}

impl ShortcutStore {
    pub fn new(file_path: PathBuf, broadcaster: Sender<Vec<Shortcut>>) -> Self {
        // Create the directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create directories for shortcuts");
            }
        }

        // Load existing shortcuts from the file
        let shortcuts = if file_path.exists() {
            let file = File::open(&file_path).expect("Failed to open shortcuts file");
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        };

        Self {
            shortcuts: Mutex::new(shortcuts),
            file_path,
            broadcaster,
        }
    }

    pub fn save(&self) {
        let shortcuts = self.shortcuts.lock().unwrap();
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)
            .expect("Failed to open shortcuts file for writing");
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &*shortcuts).expect("Failed to write shortcuts");
    }

    pub fn get_shortcuts(&self) -> Vec<Shortcut> {
        let shortcuts = self.shortcuts.lock().unwrap();
        shortcuts.clone()
    }

    // New method to broadcast the updated shortcuts list
    pub fn broadcast_shortcuts(&self) {
        let shortcuts = self.get_shortcuts();
        // Send the updated list to all subscribers
        if let Err(e) = self.broadcaster.send(shortcuts) {
            eprintln!("Error broadcasting shortcuts: {}", e);
        }
    }
}

// Shortcut-related Tauri commands

/// Retrieves the list of all shortcuts.
///
/// # Arguments
///
/// * `store` - Shared state containing the shortcuts.
///
/// # Returns
///
/// * `Result<Vec<Shortcut>, String>` - A vector of shortcuts or an error message.
#[tauri::command]
pub fn get_shortcuts_command(store: State<Arc<ShortcutStore>>) -> Result<Vec<Shortcut>, String> {
    let shortcuts = store.get_shortcuts();
    Ok(shortcuts)
}

/// Updates an existing shortcut.
///
/// # Arguments
///
/// * `shortcut` - The shortcut to update.
/// * `store` - Shared state containing the shortcuts.
/// * `app_handle` - Handle to emit events to the frontend.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if successful, Err with an error message otherwise.
#[tauri::command]
pub fn update_shortcut(
    shortcut: Shortcut,
    store: State<Arc<ShortcutStore>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut shortcuts = store.shortcuts.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = shortcuts.iter_mut().find(|s| s.id == shortcut.id) {
            existing.sequence = shortcut.sequence.clone();
        } else {
            return Err("Shortcut not found".into());
        }
    }

    store.save();

    // Broadcast the updated shortcuts list
    store.broadcast_shortcuts();

    // Emit an event to notify frontend about the update
    app_handle
        .emit_all("shortcuts_updated", store.get_shortcuts())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Adds a new shortcut.
///
/// # Arguments
///
/// * `shortcut` - The shortcut to add.
/// * `store` - Shared state containing the shortcuts.
/// * `app_handle` - Handle to emit events to the frontend.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if successful, Err with an error message otherwise.
#[tauri::command]
pub fn add_shortcut(
    mut shortcut: Shortcut,
    store: State<Arc<ShortcutStore>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut shortcuts = store.shortcuts.lock().map_err(|e| e.to_string())?;

        // Generate a unique ID based on the current time
        shortcut.id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        shortcuts.push(shortcut.clone());
    }

    store.save();

    // Broadcast the updated shortcuts list
    store.broadcast_shortcuts();

    // Emit an event to notify frontend about the addition
    app_handle
        .emit_all("shortcuts_updated", store.get_shortcuts())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Deletes an existing shortcut by ID.
///
/// # Arguments
///
/// * `id` - The ID of the shortcut to delete.
/// * `store` - Shared state containing the shortcuts.
/// * `app_handle` - Handle to emit events to the frontend.
///
/// # Returns
///
/// * `Result<(), String>` - Ok if successful, Err with an error message otherwise.
#[tauri::command]
pub fn delete_shortcut(
    id: u64,
    store: State<Arc<ShortcutStore>>,
    app_handle: AppHandle,
) -> Result<(), String> {
    {
        let mut shortcuts = store.shortcuts.lock().map_err(|e| e.to_string())?;

        if let Some(pos) = shortcuts.iter().position(|s| s.id == id) {
            shortcuts.remove(pos);
        } else {
            return Err("Shortcut not found".into());
        }
    }

    store.save();

    // Broadcast the updated shortcuts list
    store.broadcast_shortcuts();

    // Emit an event to notify frontend about the deletion
    app_handle
        .emit_all("shortcuts_updated", store.get_shortcuts())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Simulates a keyboard shortcut based on the provided keys.
///
/// # Arguments
///
/// * `shortcut_keys` - A string representing the keyboard shortcut keys (e.g., "Ctrl+S").
///
/// # Returns
///
/// * `Result<(), String>` - Ok if successful, Err with an error message otherwise.
#[tauri::command]
pub fn simulate_shortcut(sequence: Vec<String>, interval_ms: Option<u64>) -> Result<(), String> {
    println!("Simulating shortcut sequence: {:?}", sequence);

    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    // Create Enigo instance (keeping the initialization as it was)
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    let interval = std::time::Duration::from_millis(interval_ms.unwrap_or(100)); // Default interval is 100ms

    for shortcut_keys in sequence {
        println!("Simulating shortcut: {}", shortcut_keys);

        // Keep track of pressed modifiers
        let mut pressed_modifiers = vec![];

        // Split the shortcut keys and trim whitespace
        let keys: Vec<&str> = shortcut_keys.split('+').map(|k| k.trim()).collect();

        // Press down modifier keys first
        for key in &keys {
            let result = match *key {
                "Ctrl" | "Control" => enigo
                    .key(Key::Control, Direction::Press)
                    .map(|_| pressed_modifiers.push(Key::Control)),
                "Alt" => enigo
                    .key(Key::Alt, Direction::Press)
                    .map(|_| pressed_modifiers.push(Key::Alt)),
                "Shift" => enigo
                    .key(Key::Shift, Direction::Press)
                    .map(|_| pressed_modifiers.push(Key::Shift)),
                "Cmd" | "Command" | "Meta" => enigo
                    .key(Key::Meta, Direction::Press)
                    .map(|_| pressed_modifiers.push(Key::Meta)),
                _ => Ok(()),
            };

            if let Err(e) = result {
                eprintln!("Error pressing key {}: {}", key, e);
            }
        }

        // Press the main key(s)
        for key in &keys {
            if !["Ctrl", "Control", "Alt", "Shift", "Cmd", "Command", "Meta"].contains(&key) {
                let key_str = key.trim();
                let result = match key_str {
                    "Enter" => enigo.key(Key::Return, Direction::Click),
                    "Tab" => enigo.key(Key::Tab, Direction::Click),
                    "Backspace" => enigo.key(Key::Backspace, Direction::Click),
                    "Space" => enigo.key(Key::Space, Direction::Click),
                    // Add other special keys as needed
                    _ => {
                        // Handle character keys
                        let character = key_str.chars().next().unwrap();
                        let mut need_shift = false;
                        let mut char_to_use = character;

                        // Check if character is uppercase or requires Shift
                        if character.is_uppercase() || is_special_character(character) {
                            need_shift = true;
                            char_to_use = character.to_ascii_lowercase();
                        }

                        // Press Shift if needed and not already pressed
                        if need_shift && !pressed_modifiers.contains(&Key::Shift) {
                            enigo
                                .key(Key::Shift, Direction::Press)
                                .map(|_| pressed_modifiers.push(Key::Shift))
                                .map_err(|e| format!("Error pressing Shift key: {}", e))?;
                        }

                        enigo.key(Key::Unicode(char_to_use), Direction::Click)
                    }
                };

                if let Err(e) = result {
                    eprintln!("Error pressing key {}: {}", key_str, e);
                }
            }
        }

        // Release modifier keys in reverse order
        for key in pressed_modifiers.iter().rev() {
            if let Err(e) = enigo.key(*key, Direction::Release) {
                eprintln!("Error releasing key {:?}: {}", key, e);
            }
        }

        // Wait for the specified interval before the next shortcut
        std::thread::sleep(interval);
    }

    Ok(())
}

// Helper function to check if a character is a special character that requires Shift
fn is_special_character(c: char) -> bool {
    match c {
        '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '_' | '+' | '{' | '}' | '|'
        | ':' | '"' | '<' | '>' | '?' => true,
        _ => false,
    }
}
