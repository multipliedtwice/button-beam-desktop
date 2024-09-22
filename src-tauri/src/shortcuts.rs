use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State}; // Import Manager trait
use tokio::sync::broadcast::Sender;

#[derive(Serialize, Deserialize, Clone)]
pub struct Shortcut {
    pub id: u64,
    pub keys: String,
}

pub struct ShortcutStore {
    pub shortcuts: Mutex<Vec<Shortcut>>,
    pub file_path: PathBuf,
    pub broadcaster: Sender<Shortcut>,
}

impl ShortcutStore {
    pub fn new(file_path: PathBuf, broadcaster: Sender<Shortcut>) -> Self {
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
            println!("Updating shortcut with ID: {}", shortcut.id); // Debugging log
            existing.keys = shortcut.keys.clone();
        } else {
            println!("Shortcut with ID {} not found!", shortcut.id); // Debugging log
            return Err("Shortcut not found".into());
        }
    }

    store.save();

    // Broadcast the updated shortcut
    store
        .broadcaster
        .send(shortcut)
        .map_err(|e| e.to_string())?;

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

        shortcut.id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        shortcuts.insert(0, shortcut.clone());
    }

    store.save();

    // Broadcast the new shortcut
    store
        .broadcaster
        .send(shortcut.clone())
        .map_err(|e| e.to_string())?;

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
            println!("Shortcut with ID {} deleted successfully", id);
        } else {
            println!("Shortcut with ID {} not found", id);
            return Err("Shortcut not found".into());
        }
    }

    store.save();

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
pub fn simulate_shortcut(shortcut_keys: String) -> Result<(), String> {
    println!("Simulating shortcut: {}", shortcut_keys);

    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    // Create Enigo instance
    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;

    // Press down keys
    for key in shortcut_keys.split('+') {
        let result = match key.trim() {
            "Ctrl" => enigo.key(Key::Control, Direction::Press),
            "Alt" => enigo.key(Key::Alt, Direction::Press),
            "Shift" => enigo.key(Key::Shift, Direction::Press),
            "Cmd" | "Command" => enigo.key(Key::Meta, Direction::Press),
            key_str => {
                // Handle other keys, assuming they are single characters
                let character = key_str.chars().next().unwrap();
                enigo.key(Key::Unicode(character), Direction::Press)
            }
        };

        if let Err(e) = result {
            eprintln!("Error pressing key {}: {}", key, e);
        }
    }

    // Release keys in reverse order
    for key in shortcut_keys.split('+').rev() {
        let result = match key.trim() {
            "Ctrl" => enigo.key(Key::Control, Direction::Release),
            "Alt" => enigo.key(Key::Alt, Direction::Release),
            "Shift" => enigo.key(Key::Shift, Direction::Release),
            "Cmd" | "Command" => enigo.key(Key::Meta, Direction::Release),
            key_str => {
                let character = key_str.chars().next().unwrap();
                enigo.key(Key::Unicode(character), Direction::Release)
            }
        };

        if let Err(e) = result {
            eprintln!("Error releasing key {}: {}", key, e);
        }
    }

    Ok(())
}
