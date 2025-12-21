use std::{fs, path::PathBuf, sync::Mutex};

use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, FilePath};

use crate::config::AppConfig;

pub struct ConfigState(pub Mutex<AppConfig>);

fn config_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .expect("failed to resolve app config dir");

    let _ = fs::create_dir_all(&dir);
    dir.join("config.json")
}

#[tauri::command]
pub fn load_config(
    app: AppHandle,
    state: State<ConfigState>,
) -> Result<AppConfig, String> {
    let path = config_path(&app);

    let cfg = if path.exists() {
        let contents = fs::read_to_string(&path)
            .map_err(|e| e.to_string())?;

        serde_json::from_str::<AppConfig>(&contents)
            .map_err(|e| e.to_string())?
    } else {
        AppConfig::default()
    };

    let cfg = cfg.sanitize();
    *state.0.lock().unwrap() = cfg.clone();

    Ok(cfg)
}

#[tauri::command]
pub fn save_config(
    app: AppHandle,
    state: State<ConfigState>,
    cfg: AppConfig,
) -> Result<(), String> {
    let cfg = cfg.sanitize();
    let path = config_path(&app);

    let json = serde_json::to_string_pretty(&cfg)
        .map_err(|e| e.to_string())?;

    fs::write(&path, json)
        .map_err(|e| e.to_string())?;

    *state.0.lock().unwrap() = cfg;
    Ok(())
}

#[tauri::command]
pub fn read_theme_file(path: String) -> Result<String, String> {
    fs::read_to_string(path)
        .map_err(|e| e.to_string())
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