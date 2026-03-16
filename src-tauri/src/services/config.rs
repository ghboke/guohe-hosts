use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::AppConfig;

const CONFIG_FILE: &str = "config.json";
const LEGACY_UPDATE_STATE_FILE: &str = "update-state.json";

fn config_path() -> PathBuf {
    exe_dir().join(CONFIG_FILE)
}

fn legacy_update_state_path() -> PathBuf {
    exe_dir().join(LEGACY_UPDATE_STATE_FILE)
}

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyUpdateState {
    install_id: String,
    last_check_at: Option<DateTime<Utc>>,
    min_check_interval_seconds: u64,
}

pub struct ConfigService;

impl ConfigService {
    pub fn load() -> AppConfig {
        let path = config_path();
        let mut config = if !path.exists() {
            AppConfig::default()
        } else {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => AppConfig::default(),
            }
        };

        let mut changed = false;
        if config.install_id.trim().is_empty() {
            config.install_id = AppConfig::default().install_id;
            changed = true;
        }
        if config.min_update_check_interval_seconds == 0 {
            config.min_update_check_interval_seconds = AppConfig::default().min_update_check_interval_seconds;
            changed = true;
        }

        if Self::migrate_legacy_update_state(&mut config) {
            changed = true;
        }

        if changed {
            let _ = Self::save(&config);
        }

        config
    }

    pub fn save(config: &AppConfig) -> Result<(), AppError> {
        let path = config_path();
        let content = serde_json::to_string_pretty(config)
            .map_err(|e| AppError::WriteFile(format!("Serialize config: {}", e)))?;
        fs::write(&path, content)
            .map_err(|e| AppError::WriteFile(format!("Write config: {}", e)))?;
        Ok(())
    }

    fn migrate_legacy_update_state(config: &mut AppConfig) -> bool {
        let path = legacy_update_state_path();
        if !path.exists() {
            return false;
        }

        let Ok(content) = fs::read_to_string(&path) else {
            return false;
        };
        let Ok(state) = serde_json::from_str::<LegacyUpdateState>(&content) else {
            return false;
        };

        if !state.install_id.trim().is_empty() {
            config.install_id = state.install_id;
        }
        if state.last_check_at.is_some() {
            config.last_update_check_at = state.last_check_at;
        }
        if state.min_check_interval_seconds > 0 {
            config.min_update_check_interval_seconds = state.min_check_interval_seconds;
        }

        let _ = fs::remove_file(&path);
        true
    }
}
