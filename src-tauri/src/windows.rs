use std::{error::Error, fs, path::PathBuf};

use tauri::{path::BaseDirectory, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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

pub fn create_main_window(app: &AppHandle) -> Result<(), Box<dyn Error>> {
    let init_script = load_injection_script(app)?;

    WebviewWindowBuilder::new(
        app,
        "main",
        WebviewUrl::External("https://discord.com/app".parse()?),
    )
    .title("Ghostcord Lite")
    .inner_size(1100.0, 780.0)
    .resizable(true)
    .initialization_script(init_script)
    .build()?;

    Ok(())
}
