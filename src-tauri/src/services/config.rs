use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::models::AppConfig;

const CONFIG_FILE: &str = "config.json";

fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join(CONFIG_FILE)
}

pub struct ConfigService;

impl ConfigService {
    pub fn load() -> AppConfig {
        let path = config_path();
        if !path.exists() {
            return AppConfig::default();
        }
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => AppConfig::default(),
        }
    }

    pub fn save(config: &AppConfig) -> Result<(), AppError> {
        let path = config_path();
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::WriteFile(format!("Serialize config: {}", e)))?;
        fs::write(&path, content)
            .map_err(|e| AppError::WriteFile(format!("Write config: {}", e)))?;
        Ok(())
    }
}
