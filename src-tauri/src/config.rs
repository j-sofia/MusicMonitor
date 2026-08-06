use std::{fs, path::PathBuf};

use tauri::{AppHandle, Manager};

use crate::models::AppSettings;

const SETTINGS_FILE: &str = "settings.json";

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve app data directory: {error}"))?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("Could not create app data directory: {error}"))?;
    Ok(directory.join(SETTINGS_FILE))
}

pub fn load(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };
    let Ok(contents) = fs::read_to_string(path) else {
        return AppSettings::default();
    };
    serde_json::from_str::<AppSettings>(&contents)
        .map(AppSettings::sanitize)
        .unwrap_or_default()
}

pub fn save(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let temporary = path.with_extension("json.new");
    let bytes = serde_json::to_vec_pretty(settings)
        .map_err(|error| format!("Could not serialize settings: {error}"))?;
    fs::write(&temporary, bytes).map_err(|error| format!("Could not write settings: {error}"))?;

    if path.exists() {
        fs::remove_file(&path).map_err(|error| format!("Could not replace settings: {error}"))?;
    }
    fs::rename(&temporary, &path).map_err(|error| format!("Could not finalize settings: {error}"))
}
