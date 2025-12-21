use std::fs;

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::{config::AppConfig, settings, state};

#[tauri::command]
pub fn load_config(
    app: AppHandle,
    store: State<settings::SettingsStore>,
) -> Result<AppConfig, String> {
    settings::load_settings(&app, &store)
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    store: State<settings::SettingsStore>,
    cfg: AppConfig,
) -> Result<(), String> {
    let _ = settings::save_settings(&app, &store, cfg)?;
    Ok(())
}

#[tauri::command]
pub fn get_settings(
    store: State<settings::SettingsStore>,
) -> Result<AppConfig, String> {
    Ok(store.get())
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    store: State<settings::SettingsStore>,
    cfg: AppConfig,
) -> Result<AppConfig, String> {
    let cfg = settings::save_settings(&app, &store, cfg)?;
    let _ = apply_config_to_main(app, cfg.clone());
    Ok(cfg)
}

#[tauri::command]
pub fn load_state(
    app: AppHandle,
    store: State<state::StateStore>,
) -> Result<state::AppState, String> {
    state::load_state(&app, &store)
}

#[tauri::command]
pub fn save_state(
    app: AppHandle,
    store: State<state::StateStore>,
    app_state: state::AppState,
) -> Result<(), String> {
    state::save_state(&app, &store, app_state)
}

#[tauri::command]
pub fn pick_theme_file(app: AppHandle) -> Option<String> {
    let (tx, rx) = std::sync::mpsc::channel();

    app.dialog()
        .file()
        .add_filter("Theme", &["css", "theme.css"])
        .pick_file(move |file_path| {
            let _ = tx.send(file_path);
        });

    rx.recv()
        .ok()
        .flatten()
        .and_then(|p| match p {
            FilePath::Path(path) => Some(path.to_string_lossy().to_string()),
            FilePath::Url(url) => Some(url.to_string()),
        })
}

#[tauri::command]
pub fn apply_config_to_main(app: AppHandle, cfg: AppConfig) -> Result<(), String> {
    let mut cfg = cfg.sanitize();
    if cfg.theme_css.is_none() {
        if let Some(path) = cfg.theme_path.as_deref() {
            if let Ok(contents) = fs::read_to_string(path) {
                cfg.theme_css = Some(contents);
            }
        }
    }
    let window = app
        .get_webview_window("main")
        .ok_or("main window not found")?;
    let payload = serde_json::to_string(&cfg)
        .map_err(|e| e.to_string())?;
    let script = format!(
        r#"
(() => {{
  try {{
    const cfg = {payload};
    if (window.__GHOSTCORD_APPLY_CONFIG__) {{
      window.__GHOSTCORD_APPLY_CONFIG__(cfg);
    }}
  }} catch (e) {{
    console.warn("[Ghostcord] apply_config_to_main failed", e);
  }}
}})();
"#
    );
    window.eval(&script).map_err(|e| e.to_string())?;
    Ok(())
}
