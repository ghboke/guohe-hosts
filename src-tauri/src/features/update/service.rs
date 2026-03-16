use std::process::Command;
use std::time::Duration;

use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::services::ConfigService;

use super::model::UpdateCheckResult;

const UPDATE_CHECK_URL: &str = "https://rj.lovestu.com/api/v1/update/check";
const UPDATE_API_KEY: &str = "be38e79abc50a50b136863989d71c5420f5f23c14960e720";
const PRODUCT: &str = "ghhost";
const CHANNEL: &str = "stable";
const CURRENT_VERSION_CODE: u64 = 2;
const FALLBACK_UPDATE_URL: &str = "https://www.ghxi.com/ghhost.html";

#[derive(Debug, Serialize)]
struct UpdateCheckRequest {
    product: String,
    app_version: String,
    version_code: u64,
    install_id: String,
    channel: String,
    env: UpdateEnv,
}

#[derive(Debug, Serialize)]
struct UpdateEnv {
    os: String,
    os_version: String,
    locale: String,
}

#[derive(Debug, Deserialize)]
struct UpdateApiResponse {
    data: UpdateApiData,
}

#[derive(Debug, Deserialize)]
struct UpdateApiData {
    ok: bool,
    latest: Option<UpdateLatest>,
    policy: Option<UpdatePolicy>,
    extra: Option<UpdateExtra>,
}

#[derive(Debug, Deserialize)]
struct UpdateLatest {
    version: String,
    version_code: u64,
    #[serde(default)]
    changelog: String,
    #[serde(default)]
    force_update: bool,
}

#[derive(Debug, Deserialize)]
struct UpdatePolicy {
    min_check_interval_seconds: u64,
}

#[derive(Debug, Deserialize)]
struct UpdateExtra {
    update_url: Option<String>,
}

fn current_os() -> String {
    if cfg!(target_os = "windows") {
        "Windows".to_string()
    } else {
        std::env::consts::OS.to_string()
    }
}

fn current_os_version() -> String {
    if cfg!(target_os = "windows") {
        if let Ok(output) = Command::new("cmd").args(["/C", "ver"]).output() {
            let text = String::from_utf8_lossy(&output.stdout);
            if let Ok(regex) = Regex::new(r"(\d+\.\d+\.\d+)") {
                if let Some(caps) = regex.captures(&text) {
                    if let Some(value) = caps.get(1) {
                        return value.as_str().to_string();
                    }
                }
            }
        }
    }
    "unknown".to_string()
}

pub struct UpdateService;

impl UpdateService {
    pub async fn check(locale: &str, force: bool) -> Result<UpdateCheckResult, AppError> {
        let mut config = ConfigService::load();
        let now = Utc::now();

        if !force {
            if let Some(last_check_at) = config.last_update_check_at {
                let elapsed_seconds = now.signed_duration_since(last_check_at).num_seconds();
                if elapsed_seconds >= 0
                    && elapsed_seconds < config.min_update_check_interval_seconds as i64
                {
                    return Ok(UpdateCheckResult {
                        checked: false,
                        has_update: false,
                        latest_version: None,
                        latest_version_code: None,
                        changelog: None,
                        force_update: false,
                        update_url: None,
                        min_check_interval_seconds: config.min_update_check_interval_seconds,
                    });
                }
            }
        }

        let payload = UpdateCheckRequest {
            product: PRODUCT.to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
            version_code: CURRENT_VERSION_CODE,
            install_id: config.install_id.clone(),
            channel: CHANNEL.to_string(),
            env: UpdateEnv {
                os: current_os(),
                os_version: current_os_version(),
                locale: locale.to_string(),
            },
        };

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .map_err(|e| AppError::Network(format!("Build update client: {}", e)))?;

        let response = client
            .post(UPDATE_CHECK_URL)
            .header("Content-Type", "application/json")
            .header("X-Api-Key", UPDATE_API_KEY)
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Check update: {}", e)))?
            .error_for_status()
            .map_err(|e| AppError::Network(format!("Check update: {}", e)))?;

        let result: UpdateApiResponse = response
            .json()
            .await
            .map_err(|e| AppError::Network(format!("Parse update response: {}", e)))?;

        if let Some(policy) = result.data.policy {
            config.min_update_check_interval_seconds = policy.min_check_interval_seconds.max(60);
        }
        config.last_update_check_at = Some(now);
        ConfigService::save(&config)?;

        let latest = result.data.latest;
        let update_url = result
            .data
            .extra
            .and_then(|extra| extra.update_url)
            .or_else(|| Some(FALLBACK_UPDATE_URL.to_string()));
        let latest_version = latest.as_ref().map(|item| item.version.clone());
        let latest_version_code = latest.as_ref().map(|item| item.version_code);
        let force_update = latest.as_ref().map(|item| item.force_update).unwrap_or(false);
        let changelog = latest
            .as_ref()
            .and_then(|item| (!item.changelog.trim().is_empty()).then(|| item.changelog.clone()));
        let has_update = result.data.ok
            && latest
                .as_ref()
                .map(|item| item.version_code > CURRENT_VERSION_CODE)
                .unwrap_or(false);

        Ok(UpdateCheckResult {
            checked: true,
            has_update,
            latest_version,
            latest_version_code,
            changelog,
            force_update,
            update_url,
            min_check_interval_seconds: config.min_update_check_interval_seconds,
        })
    }
}
