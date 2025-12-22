use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::settings;

const VENCORD_RELEASE_API: &str =
    "https://api.github.com/repos/Vencord/Vencord/releases/latest";
const VENCORD_PRELOAD_NAME: &str = "VencordDesktopPreload.js";
const VENCORD_MAIN_NAME: &str = "VencordDesktopMain.js";

#[derive(Debug, Serialize)]
pub struct VencordAssets {
    pub version: String,
    pub dir: String,
    pub preload_path: String,
    pub main_path: String,
    pub cached: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct VencordManifest {
    version: String,
    preload_name: String,
    main_name: String,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

fn vencord_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = settings::state_path(app)?
        .parent()
        .ok_or("failed to resolve app data dir")?
        .join("vencord");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn manifest_path(app: &AppHandle) -> Result<PathBuf, String> {
    vencord_dir(app).map(|dir| dir.join("manifest.json"))
}

fn read_manifest(path: &PathBuf) -> Option<VencordManifest> {
    let contents = fs::read_to_string(path).ok()?;
    serde_json::from_str::<VencordManifest>(&contents).ok()
}

fn write_manifest(path: &PathBuf, manifest: &VencordManifest) -> Result<(), String> {
    let json = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
    fs::write(path, json).map_err(|e| e.to_string())
}

fn assets_exist(dir: &PathBuf, preload_name: &str, main_name: &str) -> bool {
    dir.join(preload_name).exists() && dir.join(main_name).exists()
}

async fn fetch_release(client: &reqwest::Client) -> Result<GithubRelease, String> {
    let response = client
        .get(VENCORD_RELEASE_API)
        .header("User-Agent", "ghostcord-lite")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response.json::<GithubRelease>().await.map_err(|e| e.to_string())
}

fn pick_asset_url(release: &GithubRelease, name: &str, fallback_key: &str) -> Option<String> {
    if let Some(asset) = release.assets.iter().find(|asset| asset.name == name) {
        return Some(asset.browser_download_url.clone());
    }

    let fallback_key = fallback_key.to_lowercase();
    release
        .assets
        .iter()
        .find(|asset| {
            let lower = asset.name.to_lowercase();
            lower.contains(&fallback_key) && lower.ends_with(".js")
        })
        .map(|asset| asset.browser_download_url.clone())
}

fn available_assets(release: &GithubRelease) -> String {
    release
        .assets
        .iter()
        .map(|asset| asset.name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

async fn download_to_path(
    client: &reqwest::Client,
    url: &str,
    path: &PathBuf,
) -> Result<(), String> {
    let response = client
        .get(url)
        .header("User-Agent", "ghostcord-lite")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("download failed for {url}: {status}"));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    fs::write(path, bytes).map_err(|e| e.to_string())
}

pub async fn ensure_vencord_assets(app: &AppHandle) -> Result<VencordAssets, String> {
    let dir = vencord_dir(app)?;
    let manifest_file = manifest_path(app)?;

    let client = reqwest::Client::new();
    let release = fetch_release(&client).await?;

    let preload_url = pick_asset_url(&release, VENCORD_PRELOAD_NAME, "desktoppreload")
        .ok_or_else(|| {
            format!(
                "missing Vencord preload asset; available: {}",
                available_assets(&release)
            )
        })?;
    let main_url = pick_asset_url(&release, VENCORD_MAIN_NAME, "desktopmain")
        .ok_or_else(|| {
            format!(
                "missing Vencord main asset; available: {}",
                available_assets(&release)
            )
        })?;

    if let Some(manifest) = read_manifest(&manifest_file) {
        if manifest.version == release.tag_name
            && assets_exist(&dir, &manifest.preload_name, &manifest.main_name)
        {
            return Ok(VencordAssets {
                version: manifest.version,
                dir: dir.to_string_lossy().to_string(),
                preload_path: dir.join(&manifest.preload_name).to_string_lossy().to_string(),
                main_path: dir.join(&manifest.main_name).to_string_lossy().to_string(),
                cached: true,
            });
        }
    }

    let preload_path = dir.join(VENCORD_PRELOAD_NAME);
    let main_path = dir.join(VENCORD_MAIN_NAME);

    download_to_path(&client, &preload_url, &preload_path).await?;
    download_to_path(&client, &main_url, &main_path).await?;

    let manifest = VencordManifest {
        version: release.tag_name.clone(),
        preload_name: VENCORD_PRELOAD_NAME.to_string(),
        main_name: VENCORD_MAIN_NAME.to_string(),
    };
    write_manifest(&manifest_file, &manifest)?;

    Ok(VencordAssets {
        version: release.tag_name,
        dir: dir.to_string_lossy().to_string(),
        preload_path: preload_path.to_string_lossy().to_string(),
        main_path: main_path.to_string_lossy().to_string(),
        cached: false,
    })
}
