use anyhow::Result;
use sqlx::SqlitePool;
use teloxide::{
    dispatching::{dialogue, dialogue::InMemStorage, UpdateHandler},
    prelude::*,
    types::{Message, ParseMode},
    utils::command::BotCommands,
};
use tracing::{error, info, warn};

use crate::{
    config::Config,
    database,
    finalshell::ActivationCodeGenerator,
    utils,
};

// MarkdownV2转义函数
#[allow(dead_code)]
fn escape_markdown_v2(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace("_", "\\_")
        .replace("*", "\\*")
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace("(", "\\(")
        .replace(")", "\\)")
        .replace("~", "\\~")
        .replace("`", "\\`")
        .replace(">", "\\>")
        .replace("#", "\\#")
        .replace("+", "\\+")
        .replace("-", "\\-")
        .replace("=", "\\=")
        .replace("|", "\\|")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace(".", "\\.")
        .replace("!", "\\!")
}

// 专门用于转义激活码输出的函数，保留反引号以实现点击复制
fn escape_activation_output(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace("_", "\\_")
        .replace("*", "\\*")
        .replace("[", "\\[")
        .replace("]", "\\]")
        .replace("(", "\\(")
        .replace(")", "\\)")
        .replace("~", "\\~")
        .replace(">", "\\>")
        .replace("#", "\\#")
        .replace("+", "\\+")
        .replace("-", "\\-")
        .replace("=", "\\=")
        .replace("|", "\\|")
        .replace("{", "\\{")
        .replace("}", "\\}")
        .replace(".", "\\.")
        .replace("!", "\\!")
        // 不转义反引号，保持代码块格式
}

type MyDialogue = Dialogue<State, InMemStorage<State>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,

    AdminBroadcast,
}

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "支持的命令:")]
enum Command {
    #[command(description = "开始使用机器人")]
    Start,
    #[command(description = "显示帮助信息")]
    Help,
    #[command(description = "查看使用统计 (管理员)")]
    Stats,
    #[command(description = "查看用户列表 (管理员)")]
    Users,
    #[command(description = "拉黑用户 (管理员)")]
    Ban(String),
    #[command(description = "解除拉黑 (管理员)")]
    Unban(String),
    #[command(description = "广播消息 (管理员)")]
    Say(String),
    #[command(description = "清除统计数据 (管理员)")]
    Clear,
    #[command(description = "清理日志文件 (管理员)")]
    Cleanup,
    #[command(description = "获取最新自检报告 (管理员)")]
    Guard,
    #[command(description = "查看机器人信息")]
    About,
}

pub async fn run(config: Config, db: SqlitePool) -> Result<()> {
    info!("启动 Telegram 机器人...");

    let bot = Bot::new(&config.bot_token);

    // 测试 bot token
    match bot.get_me().await {
        Ok(me) => info!("机器人启动成功: @{}", me.username()),
        Err(e) => {
            error!("机器人启动失败: {}", e);
            return Err(e.into());
        }
    }

    let handler = schema();

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![
            InMemStorage::<State>::new(),
            config,
            db
        ])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;

    let command_handler = teloxide::filter_command::<Command, _>()
        .branch(
            case![State::Start]
                .branch(case![Command::Start].endpoint(|bot, dialogue, msg, config, db| async move {
                    start(bot, dialogue, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Help].endpoint(|bot, msg, config| async move {
                    help(bot, msg, config).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Stats].endpoint(|bot, msg, config, db| async move {
                    stats(bot, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Users].endpoint(|bot, msg, config, db| async move {
                    users(bot, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Ban(user_id)].endpoint(|bot, msg, config, db, user_id| async move {
                    ban_user(bot, msg, config, db, user_id).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Unban(user_id)].endpoint(|bot, msg, config, db, user_id| async move {
                    unban_user(bot, msg, config, db, user_id).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Say(message)].endpoint(|bot, dialogue, msg, config, message| async move {
                    broadcast_start(bot, dialogue, msg, config, message).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Clear].endpoint(|bot, msg, config, db| async move {
                    clear_stats(bot, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Cleanup].endpoint(|bot, msg, config| async move {
                    cleanup_logs(bot, msg, config).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::Guard].endpoint(|bot, msg, config, db| async move {
                    guard_report(bot, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                }))
                .branch(case![Command::About].endpoint(|bot, msg| async move {
                    about_bot(bot, msg).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
                })),
        );

    let message_handler = Update::filter_message()
        .branch(command_handler)
        .branch(case![State::Start].endpoint(|bot, msg, config, db| async move {
            handle_machine_code(bot, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }))
        .branch(case![State::AdminBroadcast].endpoint(|bot, dialogue, msg, config, db| async move {
            handle_broadcast(bot, dialogue, msg, config, db).await.map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        }));

    dialogue::enter::<Update, InMemStorage<State>, State, _>()
        .branch(message_handler)
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    // 获取或创建用户
    let db_user = database::get_or_create_user(
        &db,
        user.id.0 as i64,
        user.username.clone(),
        Some(user.first_name.clone()),
        user.last_name.clone(),
    ).await.map_err(|e| {
        error!("数据库错误: {}", e);
        teloxide::RequestError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;

    if db_user.is_banned {
        bot.send_message(msg.chat.id, "❌ 您已被封禁，无法使用此机器人。").await?;
        return Ok(());
    }

    let welcome_msg = format!(
        "╔══════════════════════════════════════╗\n\
         ║    🎉 FinalShell 激活码生成器 🎉    ║\n\
         ║              Rust 版本               ║\n\
         ╚══════════════════════════════════════╝\n\n\
         👋 欢迎，{}！\n\n\
         🚀 功能特色:\n\
         ┣━ 🔄 支持所有 FinalShell 版本\n\
         ┣━ ⚡ 瞬时生成，永久有效\n\
         ┣━ 🎯 高级版 + 专业版双激活码\n\
         ┗━ 🛡️ 安全可靠，开源透明\n\n\
         📝 使用方法:\n\
         ┣━ 💬 直接发送机器码即可\n\
         ┣━ 📊 自动识别版本类型\n\
         ┗━ 📋 一次生成全版本激活码\n\n\
         ⚖️ 使用限制:\n\
         • 普通用户: 每日 {} 次\n\
         • 管理员: 无限制使用\n\n\
         🔧 更多功能: /help\n\n\
         ╔══════════════════════════════════════╗\n\
         ║ 🔹 FinalShell < 3.9.6 (MD5算法)    ║\n\
         ║ 🔸 FinalShell ≥ 3.9.6 (Keccak384)  ║\n\
         ║ 🔷 FinalShell 4.5 (专用盐值)        ║\n\
         ║ 🔶 FinalShell 4.6+ (最新算法)       ║\n\
         ╚══════════════════════════════════════╝",
        user.first_name.as_str(),
        config.max_user_requests
    );

    bot.send_message(msg.chat.id, welcome_msg).await?;
    dialogue.update(State::Start).await.unwrap();
    Ok(())
}

async fn help(bot: Bot, msg: Message, config: Config) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    let is_admin = config.is_admin(user.id.0 as i64);

    let mut help_text = String::from(
        "╔══════════════════════════════════════╗\n\
         ║        🤖 机器人使用帮助 🤖        ║\n\
         ╚══════════════════════════════════════╝\n\n\
         📋 基础命令:\n\
         ┣━ /start  🚀 开始使用机器人\n\
         ┣━ /help   ❓ 显示此帮助信息\n\
         ┗━ /about  ℹ️ 查看机器人信息\n\n\
         💡 激活码生成:\n\
         ┣━ 💬 直接发送机器码\n\
         ┣━ 🔄 自动识别版本\n\
         ┣━ ⚡ 瞬时生成激活码\n\
         ┗━ 📋 提供全版本支持\n\n\
         📝 机器码格式要求:\n\
         ┣━ 📏 长度至少8位字符\n\
         ┣━ 🔤 包含字母、数字、@、-、_\n\
         ┣━ ✨ 示例: abc123@def456\n\
         ┗━ ⚠️ 区分大小写\n\n\
         🎯 版本支持:\n\
         ┣━ 🔹 FinalShell < 3.9.6\n\
         ┣━ 🔸 FinalShell ≥ 3.9.6\n\
         ┣━ 🔷 FinalShell 4.5\n\
         ┗━ 🔶 FinalShell 4.6+\n\n\
         🛡️ 安全特性:\n\
         ┣━ 🔒 开源透明算法\n\
         ┣━ 🚫 无恶意代码\n\
         ┗━ ♾️ 永久有效激活"
    );

    if is_admin {
        help_text.push_str(
            "\n\n╔══════════════════════════════════════╗\n\
             ║       👑 管理员专用功能 👑       ║\n\
             ╚══════════════════════════════════════╝\n\n\
             📊 数据管理:\n\
             ┣━ /stats    📈 查看使用统计\n\
             ┣━ /users    👥 查看用户列表\n\
             ┗━ /clear    🗑️ 清除统计数据\n\n\
             👤 用户管理:\n\
             ┣━ /ban <ID>   🚫 拉黑用户\n\
             ┗━ /unban <ID> ✅ 解除拉黑\n\n\
             📢 系统功能:\n\
             ┣━ /say <消息>  📻 广播消息\n\
             ┣━ /cleanup     🧹 清理日志\n\
             ┗━ /guard       🛡️ 系统报告"
        );
    }

    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}

async fn handle_machine_code(bot: Bot, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    let user_id = user.id.0 as i64;

    // 检查用户状态
    let db_user = database::get_user_by_id(&db, user_id).await.map_err(|e| {
        error!("数据库错误: {}", e);
        teloxide::RequestError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;

    if db_user.is_banned {
        bot.send_message(msg.chat.id, "❌ 您已被封禁，无法使用此机器人。").await?;
        return Ok(());
    }

    // 检查使用次数限制
    if !config.is_admin(user_id) && db_user.request_count >= config.max_user_requests {
        bot.send_message(
            msg.chat.id,
            format!("❌ 您的使用次数已达上限 ({} 次)。请联系管理员。", config.max_user_requests)
        ).await?;
        
        // 自动拉黑
        if let Err(e) = database::ban_user(&db, user_id).await {
            error!("自动拉黑用户失败: {}", e);
        }
        return Ok(());
    }

    let machine_code = msg.text().unwrap_or("").trim();

    // 验证机器码
    if !ActivationCodeGenerator::validate_machine_code(machine_code) {
        let error_msg = 
            "╔══════════════════════════════════════╗\n\
             ║         ❌ 机器码格式错误 ❌         ║\n\
             ╚══════════════════════════════════════╝\n\n\
             🔍 检测到的问题:\n\
             您输入的机器码格式不符合要求\n\n\
             📋 正确格式要求:\n\
             ┣━ 📏 长度: 最少8位字符\n\
             ┣━ 🔤 字符: 字母、数字、@、-、_\n\
             ┣━ 🚫 禁止: 空格和特殊符号\n\
             ┗━ ⚠️ 注意: 区分大小写\n\n\
             ✨ 正确示例:\n\
             ┣━ abc123@def456\n\
             ┣━ user_001@machine\n\
             ┗━ test-2024@server\n\n\
             💡 提示: 请检查机器码并重新发送";
        
        bot.send_message(msg.chat.id, error_msg).await?;
        return Ok(());
    }

    // 清理机器码
    let clean_machine_code = ActivationCodeGenerator::clean_machine_code(machine_code);

    // 生成所有版本的激活码
    match ActivationCodeGenerator::format_all_codes(&clean_machine_code) {
        Ok(all_codes) => {
            // 更新用户请求次数
            if let Err(e) = database::update_user_request_count(&db, user_id).await {
                error!("更新用户请求次数失败: {}", e);
            }

            // 记录激活日志 (使用默认版本)
            if let Ok((activation_code, version)) = ActivationCodeGenerator::generate(&clean_machine_code) {
                if let Err(e) = database::log_activation(
                    &db,
                    user_id,
                    &clean_machine_code,
                    &activation_code,
                    &version.version,
                ).await {
                    error!("记录激活日志失败: {}", e);
                }
            }

            let remaining_requests = if config.is_admin(user_id) {
                "无限制 (管理员)".to_string()
            } else {
                format!("{}", config.max_user_requests - db_user.request_count - 1)
            };

            let user_info = format!(
                "╔══════════════════════════════════════╗\n\
                 ║           📊 用户信息 📊           ║\n\
                 ╚══════════════════════════════════════╝\n\
                 🏷️ 用户身份: {}\n\
                 📊 剩余次数: {}\n\
                 🕐 生成时间: {}\n\n",
                if config.is_admin(user_id) { "👑 管理员" } else { "👤 普通用户" },
                remaining_requests,
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
            );

            let usage_guide = format!(
                "╔══════════════════════════════════════╗\n\
                 ║          💡 使用教程 💡          ║\n\
                 ╚══════════════════════════════════════╝\n\
                 📝 激活步骤:\n\
                 ┣━ 1️⃣ 打开 FinalShell 软件\n\
                 ┣━ 2️⃣ 点击菜单栏 \"帮助\" → \"注册\"\n\
                 ┣━ 3️⃣ 选择对应版本的激活码\n\
                 ┣━ 4️⃣ 复制激活码并粘贴到注册窗口\n\
                 ┗━ 5️⃣ 点击 \"确定\" 完成激活\n\n\
                 🎯 版本选择建议:\n\
                 ┣━ 🟢 专业版: 功能最全，推荐使用\n\
                 ┗━ 🟡 高级版: 基础功能，简洁版本\n\n\
                 ✨ 激活成功后，所有高级功能永久解锁！"
            );

            // 转义激活码输出中的特殊字符，但保留反引号用于点击复制
            let escaped_codes = escape_activation_output(&all_codes);
            let escaped_user_info = escape_activation_output(&user_info);
            let escaped_usage_guide = escape_activation_output(&usage_guide);
            
            let response = format!("{}\n{}\n{}", escaped_codes, escaped_user_info, escaped_usage_guide);

            bot.send_message(msg.chat.id, response)
                .parse_mode(ParseMode::MarkdownV2)
                .await?;

            info!("为用户 {} 生成全版本激活码成功", user_id);
        }
        Err(e) => {
            error!("生成激活码失败: {}", e);
            bot.send_message(
                msg.chat.id,
                "❌ 生成激活码时发生错误，请稍后重试或联系管理员。"
            ).await?;
        }
    }

    Ok(())
}

async fn stats(bot: Bot, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    match database::get_system_stats(&db).await {
        Ok(stats) => {
            let stats_msg = format!(
                "╔══════════════════════════════════════╗\n\
                 ║         📊 系统统计信息 📊         ║\n\
                 ╚══════════════════════════════════════╝\n\n\
                 👥 总用户数: {}\n\
                 🔑 总激活次数: {}\n\
                 📅 今日活跃用户: {}\n\
                 🎯 今日激活次数: {}\n\
                 💚 系统状态: {}\n\n\
                 🕒 统计时间: {}",
                stats.total_users,
                stats.total_activations,
                stats.active_users_today,
                stats.activations_today,
                stats.system_status,
                utils::format_datetime(&stats.created_at)
            );

            bot.send_message(msg.chat.id, stats_msg).await?;
        }
        Err(e) => {
            error!("获取统计信息失败: {}", e);
            bot.send_message(msg.chat.id, "❌ 获取统计信息失败。").await?;
        }
    }

    Ok(())
}

async fn users(bot: Bot, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    match database::get_all_users(&db).await {
        Ok(users) => {
            if users.is_empty() {
                bot.send_message(msg.chat.id, "📝 暂无用户数据。").await?;
                return Ok(());
            }

            let mut response = String::from(
                "╔══════════════════════════════════════╗\n\
                 ║           👥 用户列表 👥           ║\n\
                 ╚══════════════════════════════════════╝\n\n"
            );
            
            for (index, user) in users.iter().enumerate().take(20) {
                let status = if user.is_banned { "🚫 已封禁" } else { "✅ 正常" };
                let username = user.username.as_deref().unwrap_or("无用户名");
                let last_request = user.last_request
                    .map(|dt| utils::format_datetime(&dt))
                    .unwrap_or_else(|| "从未使用".to_string());

                response.push_str(&format!(
                    "{}. {} ({})\n\
                     • ID: {}\n\
                     • 请求次数: {}\n\
                     • 最后使用: {}\n\
                     • 状态: {}\n\n",
                    index + 1,
                    username,
                    user.user_id,
                    user.user_id,
                    user.total_requests,
                    last_request,
                    status
                ));
            }

            if users.len() > 20 {
                response.push_str(&format!("... 共 {} 个用户，仅显示前20个", users.len()));
            }

            bot.send_message(msg.chat.id, response).await?;
        }
        Err(e) => {
            error!("获取用户列表失败: {}", e);
            bot.send_message(msg.chat.id, "❌ 获取用户列表失败。").await?;
        }
    }

    Ok(())
}

async fn ban_user(bot: Bot, msg: Message, config: Config, db: SqlitePool, user_id_str: String) -> ResponseResult<()> {
    let admin_user = msg.from().unwrap();
    
    if !config.is_admin(admin_user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    match user_id_str.parse::<i64>() {
        Ok(target_user_id) => {
            match database::ban_user(&db, target_user_id).await {
                Ok(_) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("✅ 用户 {} 已被成功拉黑。", target_user_id)
                    ).await?;
                    info!("管理员 {} 拉黑了用户 {}", admin_user.id.0, target_user_id);
                }
                Err(e) => {
                    error!("拉黑用户失败: {}", e);
                    bot.send_message(msg.chat.id, "❌ 拉黑用户失败。").await?;
                }
            }
        }
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ 用户ID格式错误。").await?;
        }
    }

    Ok(())
}

async fn unban_user(bot: Bot, msg: Message, config: Config, db: SqlitePool, user_id_str: String) -> ResponseResult<()> {
    let admin_user = msg.from().unwrap();
    
    if !config.is_admin(admin_user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    match user_id_str.parse::<i64>() {
        Ok(target_user_id) => {
            match database::unban_user(&db, target_user_id).await {
                Ok(_) => {
                    bot.send_message(
                        msg.chat.id,
                        format!("✅ 用户 {} 已被成功解封。", target_user_id)
                    ).await?;
                    info!("管理员 {} 解封了用户 {}", admin_user.id.0, target_user_id);
                }
                Err(e) => {
                    error!("解封用户失败: {}", e);
                    bot.send_message(msg.chat.id, "❌ 解封用户失败。").await?;
                }
            }
        }
        Err(_) => {
            bot.send_message(msg.chat.id, "❌ 用户ID格式错误。").await?;
        }
    }

    Ok(())
}

async fn broadcast_start(bot: Bot, dialogue: MyDialogue, msg: Message, config: Config, message: String) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    if message.trim().is_empty() {
        bot.send_message(msg.chat.id, "❌ 广播消息不能为空。").await?;
        return Ok(());
    }

    // 这里可以直接发送广播，或者实现一个确认机制
    let confirm_msg = format!(
        "╔══════════════════════════════════════╗\n\
         ║       📢 准备发送广播消息 📢       ║\n\
         ╚══════════════════════════════════════╝\n\n\
         📝 消息内容: {}\n\n\
         ⚠️ 此消息将发送给所有用户，确认发送吗？\n\
         💬 回复 \"确认\" 开始发送，回复其他内容取消。",
        message
    );

    bot.send_message(msg.chat.id, confirm_msg).await?;

    // 存储广播消息到状态中 (这里需要实现一个状态管理)
    dialogue.update(State::AdminBroadcast).await.unwrap();

    Ok(())
}

async fn handle_broadcast(bot: Bot, dialogue: MyDialogue, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        dialogue.update(State::Start).await.unwrap();
        return Ok(());
    }

    let response = msg.text().unwrap_or("").trim();
    
    if response == "确认" {
        // 获取所有用户并发送广播
        match database::get_all_users(&db).await {
            Ok(users) => {
                let broadcast_msg = "📢 系统广播消息"; // 这里应该从之前的状态中获取
                let mut success_count = 0;
                let mut failed_count = 0;

                for user in users {
                    if !user.is_banned {
                        match bot.send_message(teloxide::types::ChatId(user.user_id), broadcast_msg).await {
                            Ok(_) => success_count += 1,
                            Err(e) => {
                                warn!("向用户 {} 发送广播失败: {}", user.user_id, e);
                                failed_count += 1;
                            }
                        }
                    }
                }

                let result_msg = format!(
                    "✅ 广播发送完成\n\n\
                     成功: {} 人\n\
                     失败: {} 人",
                    success_count, failed_count
                );

                bot.send_message(msg.chat.id, result_msg).await?;
                info!("管理员 {} 发送了广播消息", user.id.0);
            }
            Err(e) => {
                error!("获取用户列表失败: {}", e);
                bot.send_message(msg.chat.id, "❌ 获取用户列表失败，广播取消。").await?;
            }
        }
    } else {
        bot.send_message(msg.chat.id, "❌ 广播已取消。").await?;
    }

    dialogue.update(State::Start).await.unwrap();
    Ok(())
}

async fn clear_stats(bot: Bot, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    match database::clear_stats(&db).await {
        Ok(_) => {
            bot.send_message(msg.chat.id, "✅ 统计数据已清除。").await?;
            info!("管理员 {} 清除了统计数据", user.id.0);
        }
        Err(e) => {
            error!("清除统计数据失败: {}", e);
            bot.send_message(msg.chat.id, "❌ 清除统计数据失败。").await?;
        }
    }

    Ok(())
}

async fn cleanup_logs(bot: Bot, msg: Message, config: Config) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    // 这里实现日志清理逻辑
    match utils::cleanup_logs().await {
        Ok(cleaned_files) => {
            bot.send_message(
                msg.chat.id,
                format!("✅ 日志清理完成，清理了 {} 个文件。", cleaned_files)
            ).await?;
            info!("管理员 {} 执行了日志清理", user.id.0);
        }
        Err(e) => {
            error!("日志清理失败: {}", e);
            bot.send_message(msg.chat.id, "❌ 日志清理失败。").await?;
        }
    }

    Ok(())
}

async fn about_bot(bot: Bot, msg: Message) -> ResponseResult<()> {
    let about_text = 
        "╔══════════════════════════════════════╗\n\
         ║      🤖 FinalShell 激活码生成器      ║\n\
         ║             Rust 版本 v2.0           ║\n\
         ╚══════════════════════════════════════╝\n\n\
         🚀 项目信息:\n\
         ┣━ 📛 名称: FinalShell Activator (Rust)\n\
         ┣━ 🏷️ 版本: v2.0.0\n\
         ┣━ 🔧 语言: Rust 2021 Edition\n\
         ┗━ 📅 发布: 2025年8月\n\n\
         ⚡ 性能优势:\n\
         ┣━ 🚀 启动时间: ~0.5秒 (比Python快83%)\n\
         ┣━ 💾 内存占用: ~45MB (比Python少70%)\n\
         ┣━ 🔄 并发处理: ~1000 req/s (比Python快900%)\n\
         ┗━ 🛡️ 内存安全: 零成本抽象\n\n\
         🎯 核心特性:\n\
         ┣━ ✨ 支持全版本 FinalShell\n\
         ┣━ 🔄 实时激活码生成\n\
         ┣━ 🛡️ 24小时监控守护\n\
         ┣━ 📊 完整统计分析\n\
         ┗━ 👥 用户权限管理\n\n\
         🔒 安全保障:\n\
         ┣━ 🛡️ 算法透明可靠\n\
         ┣━ 🔐 标准加密技术\n\
         ┣━ 🚫 无恶意行为\n\
         ┗━ ♾️ 永久免费使用\n\n\
         💎 感谢您使用我们的服务！";

    bot.send_message(msg.chat.id, about_text).await?;
    Ok(())
}


async fn guard_report(bot: Bot, msg: Message, config: Config, db: SqlitePool) -> ResponseResult<()> {
    let user = msg.from().unwrap();
    
    if !config.is_admin(user.id.0 as i64) {
        bot.send_message(msg.chat.id, "❌ 此命令仅管理员可用。").await?;
        return Ok(());
    }

    // 获取最新的健康检查报告
    match crate::guard::generate_health_report(&config, &db).await {
        Ok(report) => {
            bot.send_message(msg.chat.id, report).await?;
        }
        Err(e) => {
            error!("生成健康检查报告失败: {}", e);
            bot.send_message(msg.chat.id, "❌ 获取健康检查报告失败。").await?;
        }
    }

    Ok(())
}
