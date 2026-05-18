use crate::models::Settings;
use crate::paths;
use std::path::PathBuf;

fn settings_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(paths::storage_dir(app)?.join("settings.json"))
}

#[tauri::command]
pub async fn load_settings(app: tauri::AppHandle) -> Result<Settings, String> {
    let path = settings_path(&app)?;

    if !path.exists() {
        return Ok(Settings::default());
    }

    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read settings: {e}"))?;

    // Merge with defaults to handle new fields added in future versions
    let mut default_value =
        serde_json::to_value(Settings::default()).unwrap_or(serde_json::Value::Object(Default::default()));
    let loaded: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {e}"))?;

    if let (serde_json::Value::Object(ref mut defaults), serde_json::Value::Object(loaded_obj)) =
        (&mut default_value, loaded)
    {
        for (key, value) in loaded_obj {
            defaults.insert(key, value);
        }
    }

    serde_json::from_value(default_value).map_err(|e| format!("Failed to apply settings: {e}"))
}

#[tauri::command]
pub async fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app)?;

    let content =
        serde_json::to_string_pretty(&settings).map_err(|e| format!("Failed to serialize settings: {e}"))?;

    std::fs::write(&path, content).map_err(|e| format!("Failed to write settings: {e}"))?;

    Ok(())
}
