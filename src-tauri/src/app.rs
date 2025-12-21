use crate::{ipc, settings, state, windows};

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(settings::SettingsStore::new())
        .manage(state::StateStore::new())
        .invoke_handler(ipc::handler())
        .setup(|app| {
            settings::log_paths(app.handle());
            windows::create_main_window(app.handle())?;
            ipc::apply_initial_config(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Ghostcord Lite");
}
