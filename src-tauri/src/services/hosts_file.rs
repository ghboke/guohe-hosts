use std::fs;
use std::path::Path;

use crate::error::AppError;
use crate::models::HostGroup;
use crate::parser::{parse_hosts_content, serialize_groups_clean};

const DEFAULT_HOSTS_PATH: &str = r"C:\Windows\System32\drivers\etc\hosts";

pub struct HostsFileService;

impl HostsFileService {
    pub fn read(path: Option<&str>) -> Result<Vec<HostGroup>, AppError> {
        let hosts_path = path.unwrap_or(DEFAULT_HOSTS_PATH);
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
    pub fn write_clean(groups: &[HostGroup], path: Option<&str>) -> Result<(), AppError> {
        let hosts_path = path.unwrap_or(DEFAULT_HOSTS_PATH);
        let content = serialize_groups_clean(groups);

        // Create backup before writing
        let backup_path = format!("{}.bak", hosts_path);
        if Path::new(hosts_path).exists() {
            fs::copy(hosts_path, &backup_path)
                .map_err(|e| AppError::WriteFile(format!("Backup failed: {}", e)))?;
        }

        fs::write(hosts_path, content)
            .map_err(|e| AppError::WriteFile(format!("{}: {}", hosts_path, e)))?;

        Ok(())
    }
}
