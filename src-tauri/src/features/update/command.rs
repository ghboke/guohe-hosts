use tauri::State;

use crate::commands::AppState;
use crate::error::AppError;
use crate::services::ConfigService;

use super::model::UpdateCheckResult;
use super::service::UpdateService;

#[tauri::command]
pub async fn check_update(
    state: State<'_, AppState>,
    force: Option<bool>,
) -> Result<UpdateCheckResult, AppError> {
    if !super::enabled() {
        return Ok(UpdateCheckResult {
            checked: false,
            has_update: false,
            latest_version: None,
            latest_version_code: None,
            changelog: None,
            force_update: false,
            update_url: None,
            min_check_interval_seconds: 0,
        });
    }

    let locale = {
        let config = state.config.lock().unwrap();
        config.locale.clone()
    };

    let result = UpdateService::check(&locale, force.unwrap_or(false)).await?;

    let latest_config = ConfigService::load();
    let mut state_config = state.config.lock().unwrap();
    state_config.install_id = latest_config.install_id;
    state_config.last_update_check_at = latest_config.last_update_check_at;
    state_config.min_update_check_interval_seconds = latest_config.min_update_check_interval_seconds;

    Ok(result)
}
