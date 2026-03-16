use std::fs;
use std::path::Path;

use crate::error::AppError;
use crate::models::HostGroup;
use crate::parser::{parse_hosts_content, serialize_groups_clean};
use crate::system_paths::system_hosts_path;

pub struct HostsFileService;

impl HostsFileService {
    pub fn read(path: Option<&str>) -> Result<Vec<HostGroup>, AppError> {
        let default_hosts_path = system_hosts_path();
        let hosts_path = path.unwrap_or(&default_hosts_path);
        let p = Path::new(hosts_path);

        if !p.exists() {
            return Err(AppError::ReadFile(format!(
                "Hosts file not found: {}",
                hosts_path
            )));
        }

        let content =
            fs::read_to_string(p).map_err(|e| AppError::ReadFile(format!("{}: {}", hosts_path, e)))?;

        parse_hosts_content(&content)
    }

    /// Write clean hosts file (no group markers, only enabled entries).
    /// Returns whether the system hosts content was changed.
    pub fn write_clean(groups: &[HostGroup], path: Option<&str>) -> Result<bool, AppError> {
        let default_hosts_path = system_hosts_path();
        let hosts_path = path.unwrap_or(&default_hosts_path);
        let content = serialize_groups_clean(groups);
        let hosts_file = Path::new(hosts_path);

        if hosts_file.exists() {
            let current_content = fs::read_to_string(hosts_file)
                .map_err(|e| AppError::ReadFile(format!("{}: {}", hosts_path, e)))?;
            if current_content == content {
                return Ok(false);
            }
        }

        // Create backup before writing changed content
        let backup_path = format!("{}.bak", hosts_path);
        if hosts_file.exists() {
            fs::copy(hosts_path, &backup_path)
                .map_err(|e| AppError::WriteFile(format!("Backup failed: {}", e)))?;
        }

        fs::write(hosts_path, content)
            .map_err(|e| AppError::WriteFile(format!("{}: {}", hosts_path, e)))?;

        Ok(true)
    }
}
