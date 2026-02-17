use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostEntry {
    pub id: String,
    pub ip: String,
    pub hostname: String,
    pub comment: String,
    pub enabled: bool,
}

impl HostEntry {
    pub fn new(ip: String, hostname: String, comment: String, enabled: bool) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            ip,
            hostname,
            comment,
            enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostGroup {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub entries: Vec<HostEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HostGroup {
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            enabled: true,
            entries: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Build a HostGroup from metadata + parsed entries.
    pub fn from_meta(meta: GroupMeta, entries: Vec<HostEntry>) -> Self {
        Self {
            id: meta.id,
            name: meta.name,
            enabled: meta.enabled,
            entries,
            created_at: meta.created_at,
            updated_at: meta.updated_at,
        }
    }

    /// Extract metadata (without entries).
    pub fn to_meta(&self) -> GroupMeta {
        GroupMeta {
            id: self.id.clone(),
            name: self.name.clone(),
            enabled: self.enabled,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Group metadata stored in manifest.json (no entries).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupMeta {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub hosts_path: String,
    pub auto_save: bool,
    pub theme: String,
    pub locale: String,
    pub flush_dns_on_save: bool,
    #[serde(default)]
    pub active_group_id: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            hosts_path: r"C:\Windows\System32\drivers\etc\hosts".to_string(),
            auto_save: false,
            theme: "system".to_string(),
            locale: "zh-CN".to_string(),
            flush_dns_on_save: true,
            active_group_id: String::new(),
        }
    }
}
