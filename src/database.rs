use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePool, Row, SqlitePool as Pool};
use tracing::{info, warn};

use crate::models::{ActivationLog, SystemStats, User, UserStats};

pub async fn init(database_url: &str) -> Result<Pool> {
    info!("正在连接数据库: {}", database_url);
    
    let pool = SqlitePool::connect(database_url).await?;
    
    // 运行数据库迁移
    migrate(&pool).await?;
    
    Ok(pool)
}

pub async fn migrate(pool: &Pool) -> Result<()> {
    info!("运行数据库迁移...");
    
    // 创建用户表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER UNIQUE NOT NULL,
            username TEXT,
            first_name TEXT,
            last_name TEXT,
            is_admin BOOLEAN DEFAULT FALSE,
            is_banned BOOLEAN DEFAULT FALSE,
            request_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 创建激活日志表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS activation_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER NOT NULL,
            machine_code TEXT NOT NULL,
            activation_code TEXT NOT NULL,
            finalshell_version TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (user_id) REFERENCES users (user_id)
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 创建系统统计表
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS system_stats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            total_users INTEGER DEFAULT 0,
            total_activations INTEGER DEFAULT 0,
            active_users_today INTEGER DEFAULT 0,
            activations_today INTEGER DEFAULT 0,
            system_status TEXT DEFAULT 'NORMAL',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
    )
    .execute(pool)
    .await?;

    info!("数据库迁移完成");
    Ok(())
}

// 用户操作
pub async fn get_or_create_user(
    pool: &Pool,
    user_id: i64,
    username: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
) -> Result<User> {
    // 尝试获取现有用户
    if let Ok(user) = get_user_by_id(pool, user_id).await {
        return Ok(user);
    }

    // 创建新用户
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO users (user_id, username, first_name, last_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&username)
    .bind(&first_name)
    .bind(&last_name)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    get_user_by_id(pool, user_id).await
}

pub async fn get_user_by_id(pool: &Pool, user_id: i64) -> Result<User> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE user_id = ?",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn update_user_request_count(pool: &Pool, user_id: i64) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE users SET request_count = request_count + 1, updated_at = ? WHERE user_id = ?",
    )
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn ban_user(pool: &Pool, user_id: i64) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE users SET is_banned = TRUE, updated_at = ? WHERE user_id = ?",
    )
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn unban_user(pool: &Pool, user_id: i64) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE users SET is_banned = FALSE, updated_at = ? WHERE user_id = ?",
    )
    .bind(now)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_all_users(pool: &Pool) -> Result<Vec<UserStats>> {
    let users = sqlx::query(
        r#"
        SELECT 
            u.user_id,
            u.username,
            u.request_count as total_requests,
            u.is_banned,
            MAX(al.created_at) as last_request
        FROM users u
        LEFT JOIN activation_logs al ON u.user_id = al.user_id
        GROUP BY u.user_id, u.username, u.request_count, u.is_banned
        ORDER BY u.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let user_stats = users
        .into_iter()
        .map(|row| {
            let last_request = row.get::<Option<chrono::DateTime<Utc>>, _>("last_request");
            UserStats {
                user_id: row.get("user_id"),
                username: row.get("username"),
                total_requests: row.get("total_requests"),
                last_request,
                is_banned: row.get("is_banned"),
            }
        })
        .collect();

    Ok(user_stats)
}

// 激活日志操作
pub async fn log_activation(
    pool: &Pool,
    user_id: i64,
    machine_code: &str,
    activation_code: &str,
    finalshell_version: &str,
) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO activation_logs (user_id, machine_code, activation_code, finalshell_version, created_at)
        VALUES (?, ?, ?, ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(machine_code)
    .bind(activation_code)
    .bind(finalshell_version)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_activation_logs(pool: &Pool, limit: i64) -> Result<Vec<ActivationLog>> {
    let logs = sqlx::query_as::<_, ActivationLog>(
        "SELECT * FROM activation_logs ORDER BY created_at DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

// 统计操作
pub async fn get_system_stats(pool: &Pool) -> Result<SystemStats> {
    // 获取总用户数
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;

    // 获取总激活次数
    let total_activations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM activation_logs")
        .fetch_one(pool)
        .await?;

    // 获取今日活跃用户数
    let active_users_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id) FROM activation_logs WHERE DATE(created_at) = DATE('now')",
    )
    .fetch_one(pool)
    .await?;

    // 获取今日激活次数
    let activations_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activation_logs WHERE DATE(created_at) = DATE('now')",
    )
    .fetch_one(pool)
    .await?;

    Ok(SystemStats {
        id: 0,
        total_users,
        total_activations,
        active_users_today,
        activations_today,
        system_status: "NORMAL".to_string(),
        created_at: Utc::now(),
    })
}

pub async fn clear_stats(pool: &Pool) -> Result<()> {
    warn!("清除所有统计数据...");
    
    sqlx::query("DELETE FROM activation_logs")
        .execute(pool)
        .await?;
    
    sqlx::query("UPDATE users SET request_count = 0")
        .execute(pool)
        .await?;

    info!("统计数据已清除");
    Ok(())
}
