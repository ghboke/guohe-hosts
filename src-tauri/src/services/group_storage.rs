use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::models::{GroupMeta, HostEntry, HostGroup};

const DATA_DIR: &str = "data";
const MANIFEST_FILE: &str = "manifest.json";

fn data_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join(DATA_DIR)
}

fn manifest_path() -> PathBuf {
    data_dir().join(MANIFEST_FILE)
}

/// Sanitize a group name for use as a filename.
/// Replaces characters invalid on Windows (\ / : * ? " < > |) with `_`.
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Build the .hosts file path for a group name.
fn group_file_path(name: &str) -> PathBuf {
    data_dir().join(format!("{}.hosts", sanitize_filename(name)))
}

/// Serialize entries to hosts-file text.
fn serialize_entries(entries: &[HostEntry]) -> String {
    let mut lines: Vec<String> = Vec::new();
    for entry in entries {
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
    lines.join("\n")
}

pub struct GroupStorageService;

impl GroupStorageService {
    /// Check whether manifest.json exists on disk (data dir initialized).
    pub fn exists() -> bool {
        manifest_path().exists()
    }

    /// Ensure the data directory exists.
    fn ensure_data_dir() -> Result<(), AppError> {
        let dir = data_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| AppError::WriteFile(format!("Create data dir: {}", e)))?;
        }
        Ok(())
    }

    /// Load all groups: read manifest.json, then read each group's .hosts file.
    /// Returns None if manifest.json does not exist (first launch — need migration).
    pub fn load() -> Result<Option<Vec<HostGroup>>, AppError> {
        let path = manifest_path();
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| AppError::ReadFile(format!("Read manifest.json: {}", e)))?;
        let metas: Vec<GroupMeta> = serde_json::from_str(&content)
            .map_err(|e| AppError::ParseError(format!("Parse manifest.json: {}", e)))?;

        let mut groups = Vec::with_capacity(metas.len());
        for meta in metas {
            let file_path = group_file_path(&meta.name);
            let entries = if file_path.exists() {
                let text = fs::read_to_string(&file_path)
                    .map_err(|e| AppError::ReadFile(format!("Read {}.hosts: {}", meta.name, e)))?;
                Self::parse_entries(&text)?
            } else {
                Vec::new()
            };
            groups.push(HostGroup::from_meta(meta, entries));
        }

        Ok(Some(groups))
    }

    /// Save everything: manifest + all group files.
    pub fn save_all(groups: &[HostGroup]) -> Result<(), AppError> {
        Self::ensure_data_dir()?;
        Self::save_manifest(groups)?;
        for group in groups {
            Self::save_group_file(group)?;
        }
        Ok(())
    }

    /// Save only the manifest (group metadata + order). No entry files touched.
    pub fn save_manifest(groups: &[HostGroup]) -> Result<(), AppError> {
        Self::ensure_data_dir()?;
        let metas: Vec<GroupMeta> = groups.iter().map(|g| g.to_meta()).collect();
        let content = serde_json::to_string_pretty(&metas)
            .map_err(|e| AppError::WriteFile(format!("Serialize manifest: {}", e)))?;
        fs::write(manifest_path(), content)
            .map_err(|e| AppError::WriteFile(format!("Write manifest.json: {}", e)))?;
        Ok(())
    }

    /// Save a single group's .hosts file.
    pub fn save_group_file(group: &HostGroup) -> Result<(), AppError> {
        Self::ensure_data_dir()?;
        let path = group_file_path(&group.name);
        let content = serialize_entries(&group.entries);
        fs::write(&path, content)
            .map_err(|e| AppError::WriteFile(format!("Write {}.hosts: {}", group.name, e)))?;
        Ok(())
    }

    /// Delete a group's .hosts file.
    pub fn delete_group_file(name: &str) -> Result<(), AppError> {
        let path = group_file_path(name);
        if path.exists() {
            fs::remove_file(&path)
                .map_err(|e| AppError::WriteFile(format!("Delete {}.hosts: {}", name, e)))?;
        }
        Ok(())
    }

    /// Rename a group's .hosts file (old name → new name).
    pub fn rename_group_file(old_name: &str, new_name: &str) -> Result<(), AppError> {
        let old_path = group_file_path(old_name);
        let new_path = group_file_path(new_name);
        if old_path.exists() {
            fs::rename(&old_path, &new_path)
                .map_err(|e| AppError::WriteFile(format!("Rename {}.hosts → {}.hosts: {}", old_name, new_name, e)))?;
        }
        Ok(())
    }

    /// Parse entries from hosts-file text content.
    /// Reuses the same line-parsing logic as the main parser, but returns flat entries.
    fn parse_entries(content: &str) -> Result<Vec<HostEntry>, AppError> {
        let parsed = crate::parser::parse_hosts_content(content)?;
        let entries = parsed.into_iter().flat_map(|g| g.entries).collect();
        Ok(entries)
    }
}
