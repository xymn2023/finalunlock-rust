use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub chat_id: i64,
    pub admin_ids: Vec<i64>,
    pub database_url: String,
    pub max_user_requests: i32,
    pub log_level: String,
    pub guard_check_interval: u64, // 秒
}

impl Config {
    pub fn load() -> Result<Self> {
        let bot_token = env::var("BOT_TOKEN")
            .context("BOT_TOKEN 环境变量未设置")?;
        
        let chat_id = env::var("CHAT_ID")
            .context("CHAT_ID 环境变量未设置")?
            .parse::<i64>()
            .context("CHAT_ID 格式错误")?;

        let admin_ids = env::var("ADMIN_IDS")
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:finalshell_bot.db".to_string());

        let max_user_requests = env::var("MAX_USER_REQUESTS")
            .unwrap_or_else(|_| "3".to_string())
            .parse::<i32>()
            .unwrap_or(3);

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| "info".to_string());

        let guard_check_interval = env::var("GUARD_CHECK_INTERVAL")
            .unwrap_or_else(|_| "86400".to_string()) // 24小时
            .parse::<u64>()
            .unwrap_or(86400);

        Ok(Config {
            bot_token,
            chat_id,
            admin_ids,
            database_url,
            max_user_requests,
            log_level,
            guard_check_interval,
        })
    }

    pub fn is_admin(&self, user_id: i64) -> bool {
        self.admin_ids.contains(&user_id)
    }

    pub fn validate(&self) -> Result<()> {
        if self.bot_token.is_empty() {
            anyhow::bail!("Bot token 不能为空");
        }

        if self.chat_id == 0 {
            anyhow::bail!("Chat ID 不能为空");
        }

        if self.max_user_requests <= 0 {
            anyhow::bail!("最大用户请求数必须大于0");
        }

        Ok(())
    }
}
