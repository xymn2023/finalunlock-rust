-- FinalShell 激活码机器人数据库初始化脚本

-- 创建用户表
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
);

-- 创建激活日志表
CREATE TABLE IF NOT EXISTS activation_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    machine_code TEXT NOT NULL,
    activation_code TEXT NOT NULL,
    finalshell_version TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users (user_id)
);

-- 创建系统统计表
CREATE TABLE IF NOT EXISTS system_stats (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    total_users INTEGER DEFAULT 0,
    total_activations INTEGER DEFAULT 0,
    active_users_today INTEGER DEFAULT 0,
    activations_today INTEGER DEFAULT 0,
    system_status TEXT DEFAULT 'NORMAL',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 插入初始系统统计记录
INSERT OR IGNORE INTO system_stats (id, total_users, total_activations, active_users_today, activations_today, system_status) VALUES (1, 0, 0, 0, 0, 'NORMAL');

-- 优化表结构
VACUUM;

-- 完成
SELECT 'Database initialized successfully' as result;