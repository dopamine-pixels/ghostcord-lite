use tauri::{ipc::Invoke, AppHandle, Manager};

use crate::commands;

pub fn handler() -> impl Fn(Invoke<tauri::Wry>) -> bool + Send + Sync {
    tauri::generate_handler![
        commands::get_settings,
        commands::set_settings,
        commands::load_config,
        commands::save_config,
        commands::pick_theme_file,
        commands::apply_config_to_main,
        commands::load_state,
        commands::save_state,
        commands::ensure_vencord_assets,
        commands::apply_vencord_to_main,
    ]
}

pub fn apply_initial_config(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(cfg) = commands::load_config(
        app.clone(),
        app.state::<crate::settings::SettingsStore>(),
    ) {
        let _ = commands::apply_config_to_main(app.clone(), cfg);
    }

    Ok(())
}
