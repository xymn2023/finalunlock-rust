use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;
use tracing::info;

mod bot;
mod config;
mod database;
mod finalshell;
mod guard;
mod models;
mod utils;

use config::Config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 启动机器人
    Bot,
    /// 启动守护进程
    Guard,
    /// 手动执行系统检查
    Check,
    /// 初始化数据库
    InitDb,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "finalunlock_all_rust=info,teloxide=info".into()),
        )
        .init();

    // 加载环境变量
    dotenv::dotenv().ok();

    // 解析命令行参数
    let cli = Cli::parse();

    // 加载配置
    let config = Config::load()?;
    info!("配置加载成功");

    // 初始化数据库
    let db = database::init(&config.database_url).await?;
    info!("数据库初始化成功");

    match &cli.command {
        Some(Commands::Bot) => {
            info!("启动 Telegram 机器人...");
            bot::run(config, db).await?;
        }
        Some(Commands::Guard) => {
            info!("启动守护进程...");
            guard::run(config, db).await?;
        }
        Some(Commands::Check) => {
            info!("执行系统检查...");
            guard::perform_check(&config, &db).await?;
        }
        Some(Commands::InitDb) => {
            info!("初始化数据库...");
            database::migrate(&db).await?;
            info!("数据库初始化完成");
        }
        None => {
            // 默认启动机器人
            info!("启动 Telegram 机器人...");
            bot::run(config, db).await?;
        }
    }

    Ok(())
}
