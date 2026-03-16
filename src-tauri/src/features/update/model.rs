use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheckResult {
    pub checked: bool,
    pub has_update: bool,
    pub latest_version: Option<String>,
    pub latest_version_code: Option<u64>,
    pub changelog: Option<String>,
    pub force_update: bool,
    pub update_url: Option<String>,
    pub min_check_interval_seconds: u64,
}
