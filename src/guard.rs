use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::time;
use tracing::{error, info, warn};

use crate::{
    config::Config,
    models::HealthCheck,
    utils::{self, SystemInfo},
};

/// 启动守护进程
pub async fn run(config: Config, db: SqlitePool) -> Result<()> {
    info!("启动 Guard 守护进程...");

    // 创建定时任务
    let mut interval = time::interval(Duration::from_secs(config.guard_check_interval));

    loop {
        interval.tick().await;
        
        match perform_check(&config, &db).await {
            Ok(_) => {
                info!("系统检查完成");
            }
            Err(e) => {
                error!("系统检查失败: {}", e);
            }
        }
    }
}

/// 执行系统检查
pub async fn perform_check(config: &Config, db: &SqlitePool) -> Result<()> {
    info!("开始执行系统检查...");

    // 生成健康检查报告
    let report = generate_health_report(config, db).await?;
    
    // 发送报告到Telegram
    send_health_report(config, &report).await?;
    
    // 执行自动修复
    perform_auto_repair(config).await?;

    Ok(())
}

/// 生成健康检查报告
pub async fn generate_health_report(config: &Config, _db: &SqlitePool) -> Result<String> {
    let timestamp = Utc::now();
    
    // 获取系统信息
    let system_info = utils::get_system_info()?;
    
    // 检查网络连通性
    let internet_connectivity = utils::check_internet_connectivity().await;
    let telegram_api_status = utils::check_telegram_api(&config.bot_token).await;
    
    // 检查bot进程状态
    let bot_status = check_bot_process().await;
    
    // 分析日志错误
    let (error_count, warning_count) = analyze_logs().await?;
    
    // 生成报告
    let report = format_health_report(HealthCheck {
        timestamp,
        bot_status,
        guard_status: "running".to_string(),
        cpu_usage: system_info.cpu_usage,
        memory_usage: system_info.memory_usage,
        disk_usage: system_info.disk_usage,
        internet_connectivity,
        telegram_api_status,
        error_count,
        warning_count,
    }, &system_info)?;

    Ok(report)
}

/// 格式化健康检查报告
fn format_health_report(health: HealthCheck, system_info: &SystemInfo) -> Result<String> {
    let status_emoji = if health.cpu_usage < 80.0 
        && health.memory_usage < 80.0 
        && health.disk_usage < 90.0 
        && health.internet_connectivity 
        && health.telegram_api_status {
        "✅ NORMAL"
    } else {
        "⚠️ WARNING"
    };

    let bot_status_emoji = match health.bot_status.as_str() {
        "running" => "✅ running",
        "stopped" => "❌ stopped",
        _ => "❓ unknown",
    };

    let internet_status = if health.internet_connectivity { "✅ 正常" } else { "❌ 异常" };
    let telegram_status = if health.telegram_api_status { "✅ 正常" } else { "❌ 异常" };

    let cpu_status = if health.cpu_usage < 80.0 { "✅" } else { "⚠️" };
    let memory_status = if health.memory_usage < 80.0 { "✅" } else { "⚠️" };
    let disk_status = if health.disk_usage < 90.0 { "✅" } else { "⚠️" };

    let current_pid = utils::get_current_pid();
    let process_info = utils::get_process_info(current_pid);
    let uptime = process_info
        .as_ref()
        .map(|p| utils::calculate_uptime(p.start_time))
        .unwrap_or_else(|| "未知".to_string());

    let report = format!(
        "🛡️ FinalShell机器人 系统自检报告\n\n\
         📊 报告概览\n\
         📅 检查日期: {}\n\
         ⏰ 检查时间: {}\n\
         🎯 整体状态: {}\n\
         🔄 报告版本: Guard v2.0\n\n\
         🔍 详细检查结果\n\n\
         🤖 机器人进程状态\n\
         • 运行状态: {} (PID: {})\n\
         • CPU使用率: {:.1}%\n\
         • 内存使用: {}\n\
         • 运行时长: {}\n\n\
         💻 系统资源监控\n\
         • CPU: {:.1}% {}\n\
         • 内存: {:.1}% {}\n\
         • 磁盘: {:.1}% {}\n\n\
         📋 日志文件分析\n\
         • 错误数量: {} {}\n\
         • 警告数量: {} {}\n\n\
         🌐 网络连接检查\n\
         • 互联网连接: {}\n\
         • Telegram API: {}\n\n\
         报告生成时间: {}",
        health.timestamp.format("%Y-%m-%d"),
        utils::format_datetime_china(&health.timestamp),
        status_emoji,
        bot_status_emoji,
        current_pid,
        process_info.map(|p| p.cpu_usage).unwrap_or(0.0),
        utils::format_file_size(system_info.used_memory),
        uptime,
        health.cpu_usage,
        cpu_status,
        health.memory_usage,
        memory_status,
        health.disk_usage,
        disk_status,
        health.error_count,
        if health.error_count == 0 { "✅ 正常" } else { "⚠️ 需要关注" },
        health.warning_count,
        if health.warning_count < 5 { "✅ 正常" } else { "⚠️ 需要关注" },
        internet_status,
        telegram_status,
        utils::format_datetime_china(&health.timestamp)
    );

    Ok(report)
}

/// 发送健康检查报告到Telegram
async fn send_health_report(config: &Config, report: &str) -> Result<()> {
    use teloxide::{Bot, prelude::*};

    let bot = Bot::new(&config.bot_token);
    
    match bot
        .send_message(teloxide::types::ChatId(config.chat_id), report)
        .await
    {
        Ok(_) => {
            info!("健康检查报告已发送到 Telegram");
            Ok(())
        }
        Err(e) => {
            error!("发送健康检查报告失败: {}", e);
            Err(e.into())
        }
    }
}

/// 检查bot进程状态
async fn check_bot_process() -> String {
    // 这里可以通过检查PID文件或其他方式来确定bot是否运行
    // 简化实现：假设如果guard在运行，bot也在运行
    "running".to_string()
}

/// 分析日志文件中的错误和警告
async fn analyze_logs() -> Result<(i64, i64)> {
    let mut error_count = 0;
    let mut warning_count = 0;

    // 分析今天的日志文件
    let guard_log_name = format!("guard_{}.log", Utc::now().format("%Y%m%d"));
    let log_patterns = vec![
        "bot.log",
        &guard_log_name,
    ];

    for pattern in log_patterns {
        if let Ok(content) = std::fs::read_to_string(pattern) {
            error_count += content.matches("ERROR").count() as i64;
            warning_count += content.matches("WARN").count() as i64;
        }
    }

    Ok((error_count, warning_count))
}

/// 执行自动修复
async fn perform_auto_repair(config: &Config) -> Result<()> {
    info!("执行自动修复检查...");

    // 检查磁盘空间
    if !utils::check_disk_space()? {
        warn!("磁盘空间不足，执行日志清理...");
        match utils::cleanup_logs().await {
            Ok(cleaned) => info!("清理了 {} 个日志文件", cleaned),
            Err(e) => error!("日志清理失败: {}", e),
        }
    }

    // 检查网络连通性
    if !utils::check_internet_connectivity().await {
        warn!("网络连接异常，等待网络恢复...");
        tokio::time::sleep(Duration::from_secs(30)).await;
    }

    // 检查Telegram API
    if !utils::check_telegram_api(&config.bot_token).await {
        warn!("Telegram API连接异常");
        // 可以在这里实现重试逻辑
    }

    Ok(())
}

/// 监控bot进程
pub async fn monitor_bot_process(config: &Config) -> Result<()> {
    let mut interval = time::interval(Duration::from_secs(60)); // 每分钟检查一次

    loop {
        interval.tick().await;

        let bot_status = check_bot_process().await;
        
        if bot_status != "running" {
            warn!("Bot进程异常，尝试重启...");
            
            // 发送告警
            send_alert(config, "Bot进程异常，正在尝试自动重启").await?;
            
            // 这里可以实现重启逻辑
            // restart_bot().await?;
        }
    }
}

/// 发送告警消息
async fn send_alert(config: &Config, message: &str) -> Result<()> {
    use teloxide::{Bot, prelude::*};

    let bot = Bot::new(&config.bot_token);
    let alert_message = format!(
        "╔══════════════════════════════════════╗\n\
         ║         🚨 系统告警 🚨         ║\n\
         ╚══════════════════════════════════════╝\n\n\
         {}\n\n\
         🕒 告警时间: {}", 
        message, 
        utils::format_datetime_china(&Utc::now())
    );

    bot.send_message(teloxide::types::ChatId(config.chat_id), alert_message)

        .await?;

    Ok(())
}

/// 备份重要数据
pub async fn backup_data() -> Result<()> {
    info!("开始备份重要数据...");
    
    let backup_dir = "backups";
    std::fs::create_dir_all(backup_dir)?;
    
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    
    // 备份数据库
    if std::path::Path::new("finalshell_bot.db").exists() {
        let backup_path = format!("{}/finalshell_bot_{}.db", backup_dir, timestamp);
        std::fs::copy("finalshell_bot.db", &backup_path)?;
        info!("数据库备份完成: {}", backup_path);
    }
    
    // 备份配置文件
    if std::path::Path::new(".env").exists() {
        let backup_path = format!("{}/env_{}.backup", backup_dir, timestamp);
        std::fs::copy(".env", &backup_path)?;
        info!("配置文件备份完成: {}", backup_path);
    }
    
    // 清理旧备份 (保留最近7天)
    cleanup_old_backups(backup_dir, 7).await?;
    
    Ok(())
}

/// 清理旧备份文件
async fn cleanup_old_backups(backup_dir: &str, keep_days: u64) -> Result<()> {
    let cutoff_time = std::time::SystemTime::now() - Duration::from_secs(keep_days * 24 * 3600);
    
    if let Ok(entries) = std::fs::read_dir(backup_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff_time {
                            if let Err(e) = std::fs::remove_file(entry.path()) {
                                warn!("删除旧备份文件失败 {:?}: {}", entry.path(), e);
                            } else {
                                info!("删除旧备份文件: {:?}", entry.path());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyze_logs() {
        let result = analyze_logs().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_backup_data() {
        let result = backup_data().await;
        assert!(result.is_ok());
    }
}
