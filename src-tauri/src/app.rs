use tauri::Manager;

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
            let state_store = app.state::<state::StateStore>();
            let _ = state::load_state(app.handle(), &state_store);

            let window = windows::create_main_window(app.handle())?;
            windows::restore_main_window_state(&window, &state_store);
            windows::attach_main_window_state_listeners(app.handle(), &window);

            ipc::apply_initial_config(app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error running Ghostcord Lite");
}
