use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

use crate::error::AppError;

#[tauri::command]
#[allow(deprecated)]
pub fn open_external_url(app: AppHandle, url: String) -> Result<(), AppError> {
    app.shell()
        .open(url, None)
        .map_err(|e| AppError::OpenUrl(e.to_string()))
}

#[tauri::command]
pub fn get_system_hosts_path() -> String {
    crate::system_paths::system_hosts_path()
}
