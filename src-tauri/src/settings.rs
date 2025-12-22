use std::{fs, path::PathBuf, sync::Mutex};

use tauri::{AppHandle, Emitter, Manager};

use crate::config::AppConfig;

pub const SETTINGS_CHANGED_EVENT: &str = "ghostcord://settings-changed";

pub struct SettingsStore(pub Mutex<AppConfig>);

impl SettingsStore {
    pub fn new() -> Self {
        Self(Mutex::new(AppConfig::default().sanitize()))
    }

    pub fn get(&self) -> AppConfig {
        self.0.lock().unwrap().clone()
    }

    pub fn set(&self, cfg: AppConfig) -> AppConfig {
        let cfg = cfg.sanitize();
        *self.0.lock().unwrap() = cfg.clone();
        cfg
    }
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    data_dir(app).map(|dir| dir.join("settings.json"))
}

pub fn state_path(app: &AppHandle) -> Result<PathBuf, String> {
    data_dir(app).map(|dir| dir.join("state.json"))
}

fn legacy_config_path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("config.json"))
}

fn read_settings(path: &PathBuf) -> Result<AppConfig, String> {
    let contents = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str::<AppConfig>(&contents).map_err(|e| e.to_string())
}

fn write_settings(path: &PathBuf, cfg: &AppConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

pub fn load_settings(app: &AppHandle, store: &SettingsStore) -> Result<AppConfig, String> {
    let path = settings_path(app)?;

    let cfg = if path.exists() {
        read_settings(&path)?
    } else if let Some(legacy_path) = legacy_config_path(app) {
        if legacy_path.exists() {
            read_settings(&legacy_path)?
        } else {
            AppConfig::default()
        }
    } else {
        AppConfig::default()
    };

    let cfg = store.set(cfg);
    if !path.exists() {
        let _ = write_settings(&path, &cfg);
    }

    Ok(cfg)
}

pub fn save_settings(
    app: &AppHandle,
    store: &SettingsStore,
    cfg: AppConfig,
) -> Result<AppConfig, String> {
    let cfg = store.set(cfg);
    let path = settings_path(app)?;
    write_settings(&path, &cfg)?;
    emit_settings_changed(app, &cfg);
    Ok(cfg)
}

pub fn emit_settings_changed(app: &AppHandle, cfg: &AppConfig) {
    if let Err(err) = app.emit(SETTINGS_CHANGED_EVENT, cfg) {
        log::warn!("Failed to emit settings change event: {err}");
    }
}

pub fn log_paths(app: &AppHandle) {
    match settings_path(app) {
        Ok(path) => log::info!("Settings path: {}", path.display()),
        Err(err) => log::warn!("Failed to resolve settings path: {err}"),
    }
    match state_path(app) {
        Ok(path) => log::info!("State path: {}", path.display()),
        Err(err) => log::warn!("Failed to resolve state path: {err}"),
    }
}
