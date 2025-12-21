use std::{fs, path::PathBuf, sync::Mutex};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::settings;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppState {
    pub window_bounds: Option<WindowBounds>,
    pub last_active_channel: Option<String>,
    pub updater_snooze_until: Option<String>,
}

pub struct StateStore(pub Mutex<AppState>);

impl StateStore {
    pub fn new() -> Self {
        Self(Mutex::new(AppState::default()))
    }

    pub fn get(&self) -> AppState {
        self.0.lock().unwrap().clone()
    }

    pub fn set(&self, state: AppState) -> AppState {
        *self.0.lock().unwrap() = state.clone();
        state
    }
}

fn read_state(path: &PathBuf) -> Result<AppState, String> {
    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<AppState>(&contents).map_err(|e| e.to_string())
}

fn write_state(path: &PathBuf, state: &AppState) -> Result<(), String> {
    let json = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_state(app: &AppHandle, store: &StateStore) -> Result<AppState, String> {
    let path = settings::state_path(app)?;

    let state = if path.exists() {
        read_state(&path)?
    } else {
        AppState::default()
    };

    let state = store.set(state);
    if !path.exists() {
        let _ = write_state(&path, &state);
    }

    Ok(state)
}

pub fn save_state(
    app: &AppHandle,
    store: &StateStore,
    state: AppState,
) -> Result<(), String> {
    let state = store.set(state);
    let path = settings::state_path(app)?;
    write_state(&path, &state)
}
