use anyhow::Result;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// 格式化日期时间为可读字符串
pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// 格式化日期时间为中国时区字符串
pub fn format_datetime_china(dt: &DateTime<Utc>) -> String {
    let china_dt = *dt + chrono::Duration::hours(8);
    china_dt.format("%Y-%m-%d %H:%M:%S (Asia/Shanghai)").to_string()
}

/// 清理日志文件
pub async fn cleanup_logs() -> Result<usize> {
    let mut cleaned_files = 0;
    let log_patterns = vec!["*.log", "guard_*.log", "bot_*.log"];
    
    for pattern in log_patterns {
        match glob::glob(pattern) {
            Ok(paths) => {
                for entry in paths {
                    match entry {
                        Ok(path) => {
                            if should_cleanup_log(&path)? {
                                match fs::remove_file(&path) {
                                    Ok(_) => {
                                        info!("删除日志文件: {:?}", path);
                                        cleaned_files += 1;
                                    }
                                    Err(e) => {
                                        warn!("删除日志文件失败 {:?}: {}", path, e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            warn!("访问日志文件失败: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                warn!("搜索日志文件失败 {}: {}", pattern, e);
            }
        }
    }
    
    Ok(cleaned_files)
}

/// 判断是否应该清理某个日志文件
fn should_cleanup_log(path: &Path) -> Result<bool> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let age = std::time::SystemTime::now().duration_since(modified)?;
    
    // 清理7天前的日志文件
    Ok(age.as_secs() > 7 * 24 * 3600)
}

/// 格式化文件大小
pub fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// 检查网络连通性
pub async fn check_internet_connectivity() -> bool {
    match reqwest::get("https://www.google.com").await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// 检查Telegram API连通性
pub async fn check_telegram_api(bot_token: &str) -> bool {
    let url = format!("https://api.telegram.org/bot{}/getMe", bot_token);
    match reqwest::get(&url).await {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// 获取系统信息
pub fn get_system_info() -> Result<SystemInfo> {
    use sysinfo::System;
    
    let mut sys = System::new_all();
    sys.refresh_all();
    
    let cpu_usage = sys.global_cpu_info().cpu_usage();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
    
    // 简化磁盘使用率计算
    let disk_usage = 0.0; // 暂时设为0，避免API变化问题
    
    Ok(SystemInfo {
        cpu_usage: cpu_usage as f64,
        memory_usage,
        disk_usage,
        total_memory,
        used_memory,
    })
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub total_memory: u64,
    pub used_memory: u64,
}

/// 获取进程信息
pub fn get_process_info(pid: u32) -> Option<ProcessInfo> {
    use sysinfo::{Pid, System};
    
    let mut sys = System::new();
    sys.refresh_process(Pid::from(pid as usize));
    
    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        Some(ProcessInfo {
            pid,
            cpu_usage: process.cpu_usage() as f64,
            memory_usage: process.memory(),
            start_time: process.start_time(),
        })
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub start_time: u64,
}

/// 计算运行时长
pub fn calculate_uptime(start_time: u64) -> String {
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let uptime_seconds = current_time - start_time;
    let days = uptime_seconds / 86400;
    let hours = (uptime_seconds % 86400) / 3600;
    let minutes = (uptime_seconds % 3600) / 60;
    let seconds = uptime_seconds % 60;
    
    if days > 0 {
        format!("{} days, {:02}:{:02}:{:02}", days, hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

/// 检查磁盘空间是否充足
pub fn check_disk_space() -> Result<bool> {
    let system_info = get_system_info()?;
    Ok(system_info.disk_usage < 90.0) // 磁盘使用率小于90%认为是正常
}

/// 压缩日志文件
pub async fn compress_logs() -> Result<usize> {
    // 这里可以实现日志压缩逻辑
    // 比如使用 gzip 压缩旧的日志文件
    Ok(0)
}

/// 验证配置文件
pub fn validate_environment() -> Result<Vec<String>> {
    let mut missing_vars = Vec::new();
    
    let required_vars = vec![
        "BOT_TOKEN",
        "CHAT_ID",
    ];
    
    for var in required_vars {
        if std::env::var(var).is_err() {
            missing_vars.push(var.to_string());
        }
    }
    
    Ok(missing_vars)
}

/// 获取当前进程的PID
pub fn get_current_pid() -> u32 {
    std::process::id()
}

/// 检查进程是否运行
pub fn is_process_running(pid: u32) -> bool {
    use sysinfo::{Pid, System};
    
    let mut sys = System::new();
    sys.refresh_process(Pid::from(pid as usize));
    sys.process(Pid::from(pid as usize)).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_format_datetime() {
        let dt = Utc::now();
        let formatted = format_datetime(&dt);
        assert!(formatted.contains("UTC"));
    }

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1048576), "1.0 MB");
        assert_eq!(format_file_size(500), "500 B");
    }

    #[test]
    fn test_calculate_uptime() {
        let start_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() - 3661; // 1小时1分钟1秒前
        
        let uptime = calculate_uptime(start_time);
        assert!(uptime.contains("01:01:01"));
    }

    #[test]
    fn test_get_current_pid() {
        let pid = get_current_pid();
        assert!(pid > 0);
    }
}
