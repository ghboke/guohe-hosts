use std::sync::Mutex;

use tauri::State;

use crate::error::AppError;
use crate::models::{AppConfig, HostEntry, HostGroup};
use crate::services::{ConfigService, GroupStorageService, HostsFileService};

pub struct AppState {
    pub groups: Mutex<Vec<HostGroup>>,
    pub config: Mutex<AppConfig>,
}

// ─── Config ──────────────────────────────────────────

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Result<AppConfig, AppError> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
pub fn save_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), AppError> {
    let mut merged = config;
    {
        let state_config = state.config.lock().unwrap();
        merged.install_id = state_config.install_id.clone();
        merged.last_update_check_at = state_config.last_update_check_at;
        merged.min_update_check_interval_seconds = state_config.min_update_check_interval_seconds;
    }
    ConfigService::save(&merged)?;
    let mut state_config = state.config.lock().unwrap();
    *state_config = merged;
    Ok(())
}

// ─── Hosts file ──────────────────────────────────────

#[tauri::command]
pub fn read_hosts_file(state: State<'_, AppState>) -> Result<Vec<HostGroup>, AppError> {
    // Read config for hosts_path and active_group_id
    let (hosts_path, active_id) = {
        let config = state.config.lock().unwrap();
        (config.hosts_path.clone(), config.active_group_id.clone())
    };

    // 1. Load groups from storage or migrate from system hosts
    let mut groups = if GroupStorageService::exists() {
        GroupStorageService::load()?.unwrap_or_default()
    } else {
        Vec::new()
    };

    // If no entries at all (first launch or stale empty data), migrate from system hosts
    let has_entries = groups.iter().any(|g| !g.entries.is_empty());
    if !has_entries {
        if let Ok(from_hosts) = HostsFileService::read(Some(&hosts_path)) {
            if !from_hosts.is_empty() {
                groups = from_hosts;
                GroupStorageService::save_all(&groups)?;
            }
        }
    }

    // Ensure there's always at least one group
    if groups.is_empty() {
        groups.push(HostGroup::new("System".to_string()));
        GroupStorageService::save_all(&groups)?;
    }

    // 2. Resolve active group
    let has_active = !active_id.is_empty() && groups.iter().any(|g| g.id == active_id);

    if has_active {
        // Compare system hosts with active group; if different, system wins
        if let Ok(system_content) = std::fs::read_to_string(&hosts_path) {
            let active_group = groups.iter().find(|g| g.id == active_id).unwrap();
            let local_text = serialize_entries_for_compare(&active_group.entries);

            if let Ok(parsed) = crate::parser::parse_hosts_content(&system_content) {
                let system_entries: Vec<HostEntry> =
                    parsed.into_iter().flat_map(|g| g.entries).collect();
                let system_text = serialize_entries_for_compare(&system_entries);

                if local_text != system_text {
                    // System hosts was modified externally, update local copy
                    let active_group =
                        groups.iter_mut().find(|g| g.id == active_id).unwrap();
                    active_group.entries = system_entries;
                    active_group.updated_at = chrono::Utc::now();
                    GroupStorageService::save_group_file(active_group)?;
                    GroupStorageService::save_manifest(&groups)?;
                }
            }
        }

        // Set enabled flags to reflect active group
        for g in groups.iter_mut() {
            g.enabled = g.id == active_id;
        }
    } else {
        // No valid active group — activate the first one
        if let Some(first) = groups.first_mut() {
            first.enabled = true;
            let first_id = first.id.clone();
            for g in groups.iter_mut().skip(1) {
                g.enabled = false;
            }

            let mut config = state.config.lock().unwrap();
            config.active_group_id = first_id;
            ConfigService::save(&config)?;
        }
        GroupStorageService::save_manifest(&groups)?;
    }

    let mut state_groups = state.groups.lock().unwrap();
    *state_groups = groups.clone();
    Ok(groups)
}

/// Serialize entries to a normalized string for comparison purposes.
fn serialize_entries_for_compare(entries: &[HostEntry]) -> String {
    entries
        .iter()
        .filter(|e| e.enabled)
        .map(|e| {
            let comment = if e.comment.is_empty() {
                String::new()
            } else {
                format!(" # {}", e.comment)
            };
            format!("{} {}{}", e.ip, e.hostname, comment)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[tauri::command]
pub fn write_hosts_file(state: State<'_, AppState>) -> Result<(), AppError> {
    let groups = state.groups.lock().unwrap().clone();

    // Group metadata and .hosts files are already persisted by the edit commands.
    // Manual save only needs to sync the current active content into the system hosts file.
    let config = state.config.lock().unwrap();
    let hosts_path = config.hosts_path.clone();
    let flush_dns = config.flush_dns_on_save;
    drop(config);

    let changed = HostsFileService::write_clean(&groups, Some(&hosts_path))?;

    if changed && flush_dns {
        let _ = flush_dns_internal();
    }

    Ok(())
}

// ─── DNS flush ───────────────────────────────────────

fn flush_dns_internal() -> Result<String, AppError> {
    let output = std::process::Command::new("ipconfig")
        .arg("/flushdns")
        .output()
        .map_err(|e| AppError::Io(e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

#[tauri::command]
pub fn flush_dns() -> Result<String, AppError> {
    flush_dns_internal()
}

// ─── Backup / History ────────────────────────────────

#[tauri::command]
pub fn create_backup() -> Result<String, AppError> {
    let hosts_path = crate::system_paths::system_hosts_path();
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    let backup_dir = backup_dir()?;
    let backup_file = backup_dir.join(format!("hosts_{}.bak", timestamp));

    std::fs::copy(&hosts_path, &backup_file)
        .map_err(|e| AppError::WriteFile(format!("Backup failed: {}", e)))?;

    Ok(backup_file.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_backups() -> Result<Vec<BackupInfo>, AppError> {
    let dir = backup_dir()?;
    let mut backups = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("bak") {
                if let Ok(meta) = entry.metadata() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let modified = meta
                        .modified()
                        .ok()
                        .and_then(|t| {
                            let dt: chrono::DateTime<chrono::Local> = t.into();
                            Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        })
                        .unwrap_or_default();
                    let size = meta.len();
                    backups.push(BackupInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        modified,
                        size,
                    });
                }
            }
        }
    }

    backups.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(backups)
}

#[tauri::command]
pub fn restore_backup(
    state: State<'_, AppState>,
    backup_path: String,
) -> Result<Vec<HostGroup>, AppError> {
    let hosts_path = crate::system_paths::system_hosts_path();

    // 1. Restore hosts file from backup
    std::fs::copy(&backup_path, &hosts_path)
        .map_err(|e| AppError::WriteFile(format!("Restore failed: {}", e)))?;

    // 2. Re-parse and save to multi-file storage
    let groups = HostsFileService::read(None)?;
    GroupStorageService::save_all(&groups)?;

    // 3. Update in-memory state
    let mut state_groups = state.groups.lock().unwrap();
    *state_groups = groups.clone();

    // 4. Flush DNS
    let _ = flush_dns_internal();

    Ok(groups)
}

#[tauri::command]
pub fn delete_backup(backup_path: String) -> Result<(), AppError> {
    std::fs::remove_file(&backup_path)
        .map_err(|e| AppError::WriteFile(format!("Delete backup failed: {}", e)))?;
    Ok(())
}

fn backup_dir() -> Result<std::path::PathBuf, AppError> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let dir = exe_dir.join("backups");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::WriteFile(format!("Create backup dir: {}", e)))?;
    }
    Ok(dir)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub modified: String,
    pub size: u64,
}

// ─── Import / Export ─────────────────────────────────

#[tauri::command]
pub fn import_from_text(
    state: State<'_, AppState>,
    text: String,
    group_name: String,
) -> Result<HostGroup, AppError> {
    let parsed = crate::parser::parse_hosts_content(&text)?;
    let mut group = HostGroup::new(group_name);
    for g in parsed {
        for entry in g.entries {
            group.entries.push(entry);
        }
    }
    let mut groups = state.groups.lock().unwrap();
    groups.push(group.clone());
    GroupStorageService::save_manifest(&groups)?;
    GroupStorageService::save_group_file(&group)?;
    Ok(group)
}

#[tauri::command]
pub fn export_group(state: State<'_, AppState>, group_id: String) -> Result<String, AppError> {
    let groups = state.groups.lock().unwrap();
    let group = groups
        .iter()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;

    let mut lines = Vec::new();
    for entry in &group.entries {
        let comment_part = if entry.comment.is_empty() {
            String::new()
        } else {
            format!(" # {}", entry.comment)
        };
        if entry.enabled {
            lines.push(format!("{} {}{}", entry.ip, entry.hostname, comment_part));
        } else {
            lines.push(format!("# {} {}{}", entry.ip, entry.hostname, comment_part));
        }
    }
    Ok(lines.join("\n"))
}

// ─── Groups CRUD ─────────────────────────────────────

#[tauri::command]
pub fn get_groups(state: State<'_, AppState>) -> Result<Vec<HostGroup>, AppError> {
    let groups = state.groups.lock().unwrap();
    Ok(groups.clone())
}

#[tauri::command]
pub fn add_group(state: State<'_, AppState>, name: String) -> Result<HostGroup, AppError> {
    let mut groups = state.groups.lock().unwrap();
    let mut group = HostGroup::new(name);
    group.enabled = false; // New groups are not active by default
    groups.push(group.clone());
    GroupStorageService::save_manifest(&groups)?;
    GroupStorageService::save_group_file(&group)?;
    Ok(group)
}

#[tauri::command]
pub fn delete_group(state: State<'_, AppState>, group_id: String) -> Result<Vec<HostGroup>, AppError> {
    let mut groups = state.groups.lock().unwrap();
    let idx = groups
        .iter()
        .position(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id.clone()))?;
    let removed = groups.remove(idx);
    GroupStorageService::delete_group_file(&removed.name)?;

    // If the deleted group was the active one, activate the first remaining group
    if removed.enabled {
        if let Some(first) = groups.first_mut() {
            first.enabled = true;
        }
        let new_active_id = groups.first().map(|g| g.id.clone()).unwrap_or_default();
        GroupStorageService::save_manifest(&groups)?;
        let result = groups.clone();
        drop(groups);

        let mut config = state.config.lock().unwrap();
        config.active_group_id = new_active_id;
        ConfigService::save(&config)?;

        Ok(result)
    } else {
        GroupStorageService::save_manifest(&groups)?;
        Ok(groups.clone())
    }
}

#[tauri::command]
pub fn activate_group(
    state: State<'_, AppState>,
    group_id: String,
) -> Result<Vec<HostGroup>, AppError> {
    let groups = {
        let mut groups = state.groups.lock().unwrap();

        if !groups.iter().any(|g| g.id == group_id) {
            return Err(AppError::GroupNotFound(group_id));
        }

        // Single-select: only one group can be active
        for g in groups.iter_mut() {
            let is_target = g.id == group_id;
            g.enabled = is_target;
            if is_target {
                g.updated_at = chrono::Utc::now();
            }
        }

        GroupStorageService::save_manifest(&groups)?;
        groups.clone()
    };

    // Update config with active group id
    let (hosts_path, flush_dns) = {
        let mut config = state.config.lock().unwrap();
        config.active_group_id = group_id;
        ConfigService::save(&config)?;
        (config.hosts_path.clone(), config.flush_dns_on_save)
    };

    // Write system hosts + flush DNS in background to avoid blocking UI
    let groups_for_bg = groups.clone();
    std::thread::spawn(move || {
        if let Ok(changed) = HostsFileService::write_clean(&groups_for_bg, Some(&hosts_path)) {
            if changed && flush_dns {
                let _ = flush_dns_internal();
            }
        }
    });

    Ok(groups)
}

#[tauri::command]
pub fn rename_group(
    state: State<'_, AppState>,
    group_id: String,
    name: String,
) -> Result<HostGroup, AppError> {
    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;
    let old_name = group.name.clone();
    group.name = name.clone();
    group.updated_at = chrono::Utc::now();
    let result = group.clone();
    GroupStorageService::save_manifest(&groups)?;
    GroupStorageService::rename_group_file(&old_name, &name)?;
    Ok(result)
}

#[tauri::command]
pub fn reorder_groups(state: State<'_, AppState>, group_ids: Vec<String>) -> Result<(), AppError> {
    let mut groups = state.groups.lock().unwrap();
    let mut reordered = Vec::with_capacity(group_ids.len());
    for id in &group_ids {
        if let Some(g) = groups.iter().find(|g| &g.id == id) {
            reordered.push(g.clone());
        }
    }
    *groups = reordered;
    GroupStorageService::save_manifest(&groups)?;
    Ok(())
}

// ─── Entries CRUD ────────────────────────────────────

#[tauri::command]
pub fn add_entry(
    state: State<'_, AppState>,
    group_id: String,
    ip: String,
    hostname: String,
    comment: String,
) -> Result<HostEntry, AppError> {
    validate_ip(&ip)?;
    validate_hostname(&hostname)?;

    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;

    let entry = HostEntry::new(ip, hostname, comment, true);
    group.entries.push(entry.clone());
    group.updated_at = chrono::Utc::now();
    GroupStorageService::save_group_file(group)?;
    GroupStorageService::save_manifest(&groups)?;
    Ok(entry)
}

#[tauri::command]
pub fn update_entry(
    state: State<'_, AppState>,
    group_id: String,
    entry_id: String,
    ip: String,
    hostname: String,
    comment: String,
) -> Result<HostEntry, AppError> {
    validate_ip(&ip)?;
    validate_hostname(&hostname)?;

    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id.clone()))?;

    let entry = group
        .entries
        .iter_mut()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| AppError::EntryNotFound(entry_id))?;

    entry.ip = ip;
    entry.hostname = hostname;
    entry.comment = comment;
    group.updated_at = chrono::Utc::now();
    let result = entry.clone();
    GroupStorageService::save_group_file(group)?;
    GroupStorageService::save_manifest(&groups)?;
    Ok(result)
}

#[tauri::command]
pub fn delete_entry(
    state: State<'_, AppState>,
    group_id: String,
    entry_id: String,
) -> Result<(), AppError> {
    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;

    let idx = group
        .entries
        .iter()
        .position(|e| e.id == entry_id)
        .ok_or_else(|| AppError::EntryNotFound(entry_id))?;

    group.entries.remove(idx);
    group.updated_at = chrono::Utc::now();
    GroupStorageService::save_group_file(group)?;
    GroupStorageService::save_manifest(&groups)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_entry(
    state: State<'_, AppState>,
    group_id: String,
    entry_id: String,
    enabled: bool,
) -> Result<HostEntry, AppError> {
    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;

    let entry = group
        .entries
        .iter_mut()
        .find(|e| e.id == entry_id)
        .ok_or_else(|| AppError::EntryNotFound(entry_id))?;

    entry.enabled = enabled;
    group.updated_at = chrono::Utc::now();
    let result = entry.clone();
    GroupStorageService::save_group_file(group)?;
    GroupStorageService::save_manifest(&groups)?;
    Ok(result)
}

#[tauri::command]
pub fn reorder_entries(
    state: State<'_, AppState>,
    group_id: String,
    entry_ids: Vec<String>,
) -> Result<(), AppError> {
    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;

    let mut reordered = Vec::with_capacity(entry_ids.len());
    for id in &entry_ids {
        if let Some(e) = group.entries.iter().find(|e| &e.id == id) {
            reordered.push(e.clone());
        }
    }
    group.entries = reordered;
    group.updated_at = chrono::Utc::now();
    GroupStorageService::save_group_file(group)?;
    GroupStorageService::save_manifest(&groups)?;
    Ok(())
}

// ─── CodeMirror (raw text) ───────────────────────────

#[tauri::command]
pub fn get_group_text(state: State<'_, AppState>, group_id: String) -> Result<String, AppError> {
    let (group, hosts_path) = {
        let groups = state.groups.lock().unwrap();
        let group = groups
            .iter()
            .find(|g| g.id == group_id)
            .ok_or_else(|| AppError::GroupNotFound(group_id.clone()))?
            .clone();
        let hosts_path = if group.enabled {
            let config = state.config.lock().unwrap();
            Some(config.hosts_path.clone())
        } else {
            None
        };
        (group, hosts_path)
    };

    // In text mode, show full raw content for the currently active hosts source.
    if let Some(path) = hosts_path {
        if let Ok(text) = std::fs::read_to_string(&path) {
            return Ok(text);
        }
    }

    let mut lines = Vec::new();
    for entry in &group.entries {
        let comment_part = if entry.comment.is_empty() {
            String::new()
        } else {
            format!(" # {}", entry.comment)
        };
        if entry.enabled {
            lines.push(format!("{} {}{}", entry.ip, entry.hostname, comment_part));
        } else {
            lines.push(format!("# {} {}{}", entry.ip, entry.hostname, comment_part));
        }
    }
    Ok(lines.join("\n"))
}

#[tauri::command]
pub fn update_group_from_text(
    state: State<'_, AppState>,
    group_id: String,
    text: String,
) -> Result<HostGroup, AppError> {
    let parsed = crate::parser::parse_hosts_content(&text)?;
    let mut entries = Vec::new();
    for g in parsed {
        for entry in g.entries {
            entries.push(entry);
        }
    }

    let mut groups = state.groups.lock().unwrap();
    let group = groups
        .iter_mut()
        .find(|g| g.id == group_id)
        .ok_or_else(|| AppError::GroupNotFound(group_id))?;
    group.entries = entries;
    group.updated_at = chrono::Utc::now();
    let result = group.clone();
    GroupStorageService::save_group_file(group)?;
    GroupStorageService::save_manifest(&groups)?;
    Ok(result)
}

// ─── Validation ──────────────────────────────────────

fn validate_ip(ip: &str) -> Result<(), AppError> {
    let ip = ip.trim();
    if ip.is_empty() {
        return Err(AppError::InvalidInput("IP address cannot be empty".into()));
    }
    if ip.parse::<std::net::IpAddr>().is_err() {
        return Err(AppError::InvalidInput(format!("Invalid IP address: {}", ip)));
    }
    Ok(())
}

fn validate_hostname(hostname: &str) -> Result<(), AppError> {
    let hostname = hostname.trim();
    if hostname.is_empty() {
        return Err(AppError::InvalidInput("Hostname cannot be empty".into()));
    }
    if hostname.len() > 253 {
        return Err(AppError::InvalidInput("Hostname too long".into()));
    }
    Ok(())
}
