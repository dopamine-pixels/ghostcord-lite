use std::{error::Error, fs, path::PathBuf};

use tauri::{
    path::BaseDirectory, AppHandle, Manager, PhysicalPosition, PhysicalSize,
    Position, Size, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
};

use crate::state;

fn load_injection_script(app: &AppHandle) -> Result<String, Box<dyn Error>> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(path) = app
        .path()
        .resolve("injection/ghostcord_init.js", BaseDirectory::Resource)
    {
        candidates.push(path);
    }

    if let Ok(path) = app
        .path()
        .resolve("src/injection/ghostcord_init.js", BaseDirectory::Resource)
    {
        candidates.push(path);
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join("injection").join("ghostcord_init.js"));
        candidates.push(
            resource_dir
                .join("src")
                .join("injection")
                .join("ghostcord_init.js"),
        );
    }

    if let Ok(current_dir) = std::env::current_dir() {
        candidates.push(
            current_dir
                .join("src-tauri")
                .join("src")
                .join("injection")
                .join("ghostcord_init.js"),
        );
    }

    for path in candidates {
        if let Ok(contents) = fs::read_to_string(&path) {
            log::info!("Loaded injection script from {}", path.display());
            return Ok(contents);
        }
    }

    Err("ghostcord_init.js not found in resources or dev paths".into())
}

pub fn create_main_window(app: &AppHandle) -> Result<WebviewWindow, Box<dyn Error>> {
    let init_script = load_injection_script(app)?;

    let window = WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::External("https://discord.com/app".parse()?),
    )
    .title("Ghostcord Lite")
    .inner_size(1100.0, 780.0)
    .resizable(true)
    .initialization_script(init_script)
    .build()?;

    if cfg!(debug_assertions) {
        window.open_devtools();
    }

    Ok(window)
}

pub fn restore_main_window_state(
    window: &WebviewWindow,
    store: &state::StateStore,
) {
    if let Some(bounds) = store.get().window_bounds {
        if bounds.width > 0 && bounds.height > 0 {
            let _ = window.set_size(Size::Physical(PhysicalSize::new(
                bounds.width,
                bounds.height,
            )));
        }
        let _ = window.set_position(Position::Physical(PhysicalPosition::new(
            bounds.x,
            bounds.y,
        )));
    }
}

pub fn attach_main_window_state_listeners(
    app: &AppHandle,
    window: &WebviewWindow,
) {
    let app_handle = app.clone();
    let window = window.clone();

    window.on_window_event(move |event| {
        let store = app_handle.state::<state::StateStore>();
        match event {
            WindowEvent::Moved(pos) => {
                update_bounds(&store, Some(*pos), None);
            }
            WindowEvent::Resized(size) => {
                update_bounds(&store, None, Some(*size));
            }
            WindowEvent::CloseRequested { .. } => {
                let app_state = store.get();
                let _ = state::save_state(&app_handle, &store, app_state);
            }
            _ => {}
        }
    });

    let store = app.state::<state::StateStore>();
    if let Ok(position) = window.outer_position() {
        if let Ok(size) = window.outer_size() {
            update_bounds(&store, Some(position), Some(size));
        }
    }
}

fn update_bounds(
    store: &state::StateStore,
    position: Option<PhysicalPosition<i32>>,
    size: Option<PhysicalSize<u32>>,
) {
    let mut app_state = store.get();
    let mut bounds = app_state
        .window_bounds
        .unwrap_or(state::WindowBounds {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        });

    if let Some(pos) = position {
        bounds.x = pos.x;
        bounds.y = pos.y;
    }
    if let Some(sz) = size {
        bounds.width = sz.width;
        bounds.height = sz.height;
    }

    app_state.window_bounds = Some(bounds);
    store.set(app_state);
}
