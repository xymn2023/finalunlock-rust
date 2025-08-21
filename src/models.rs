use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub user_id: i64,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_admin: bool,
    pub is_banned: bool,
    pub request_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ActivationLog {
    pub id: i64,
    pub user_id: i64,
    pub machine_code: String,
    pub activation_code: String,
    pub finalshell_version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SystemStats {
    pub id: i64,
    pub total_users: i64,
    pub total_activations: i64,
    pub active_users_today: i64,
    pub activations_today: i64,
    pub system_status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub timestamp: DateTime<Utc>,
    pub bot_status: String,
    pub guard_status: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub internet_connectivity: bool,
    pub telegram_api_status: bool,
    pub error_count: i64,
    pub warning_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalShellVersion {
    pub version: String,
    pub is_legacy: bool,
}

impl FinalShellVersion {
    pub fn detect_version(machine_code: &str) -> Self {
        // 基于机器码长度和格式来判断版本，默认返回最新版本
        if machine_code.len() < 15 {
            FinalShellVersion {
                version: "< 3.9.6".to_string(),
                is_legacy: true,
            }
        } else if machine_code.contains("-") && machine_code.len() > 25 {
            FinalShellVersion {
                version: "4.6+".to_string(),
                is_legacy: false,
            }
        } else if machine_code.len() > 20 {
            FinalShellVersion {
                version: "≥ 3.9.6".to_string(),
                is_legacy: false,
            }
        } else {
            FinalShellVersion {
                version: "4.5".to_string(),
                is_legacy: false,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub user_id: i64,
    pub username: Option<String>,
    pub total_requests: i32,
    pub last_request: Option<DateTime<Utc>>,
    pub is_banned: bool,
}
