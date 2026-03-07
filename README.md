# 🚀 FinalShell 激活码 Telegram 机器人 (Rust 版本)

<div align="center">

**🎯 高性能 | 🛡️ 内存安全 | 📊 完整监控 | 🔧 企业级稳定性**

[快速开始](#-快速开始) • [功能特性](#-功能特性) • [Demo](https://t.me/finalunlock_rust_bot)  • [精简版](https://github.com/xymn2023/FinalUnlock)

</div>

---

## 📋 项目简介

**finalunlock-rust** 专为FinalShell激活码自动化分发而设计。采用现代化Rust架构，集成了智能守护系统、完整监控体系和自动化运维功能，确保7×24小时稳定运行。

### 🎯 核心优势

- **🚀 高性能**: Rust零成本抽象，内存安全，高并发处理
- **🛡️ 智能守护**: Guard守护程序自动监控和故障恢复  
- **📊 企业级监控**: 实时健康检查、资源监控、自动报告
- **🔧 运维自动化**: 自动重启、日志轮转、配置验证
- **🌐 全版本支持**: 支持FinalShell全版本（包括4.6+）
- **👥 用户管理**: 完整的权限控制和黑名单机制
- **⚡ 内存效率**: 相比Python版本内存占用降低70%+

---

## ⚡ 快速开始

### 📋 系统要求

- **操作系统**: Linux (Ubuntu 18.04+, Debian 9+)  
- **Rust版本**: Rust 1.70+
- **内存要求**: 最少256MB RAM
- **磁盘空间**: 最少500MB可用空间
- **网络要求**: 能够访问GitHub和Telegram API

### 🦀 安装Rust

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 📦 克隆和构建项目

```bash
# 克隆项目
git clone https://github.com/xymn2023/finalunlock-rust.git
cd finalunlock-all

# 配置环境变量
cp env.example .env
# 编辑 .env 文件，配置你的 BOT_TOKEN 和 CHAT_ID

# 构建和运行
chmod +x start.sh
./start.sh --build
./start.sh --start
```

### 🚀 一键启动

```bash
# 启动管理界面
./start.sh

# 或者命令行模式
./start.sh --start    # 启动所有服务
./start.sh --stop     # 停止所有服务
./start.sh --restart  # 重启所有服务
./start.sh --status   # 查看状态
```

---

## 🌟 功能特性

### 🤖 核心机器人功能

| 功能 | 描述 | 状态 |
|------|------|------|
| **全版本支持** | FinalShell < 3.9.6, ≥ 3.9.6, 4.5, 4.6+ | ✅ |
| **智能激活码生成** | 基于Keccak384和MD5算法生成对应版本激活码 | ✅ |
| **高级版&专业版** | 同时生成高级版和专业版激活码 | ✅ |
| **用户权限管理** | 管理员/普通用户权限分离 | ✅ |
| **使用次数限制** | 普通用户3次限制，超限自动拉黑 | ✅ |
| **黑名单机制** | 支持手动和自动拉黑/解封 | ✅ |
| **广播功能** | 管理员可向所有用户发送消息 | ✅ |
| **统计分析** | 详细的使用统计和用户分析 | ✅ |
| **数据持久化** | SQLite数据库存储，支持迁移 | ✅ |

### 🛡️ Guard 守护系统

| 功能 | 描述 | 状态 |
|------|------|------|
| **自动自检** | 每天00:00执行全面系统检查 | ✅ |
| **定时报告** | 每天00:00自检后立即发送详细Markdown报告 | ✅ |
| **进程监控** | 实时监控机器人进程状态 | ✅ |
| **资源监控** | CPU、内存、磁盘使用率监控 | ✅ |
| **网络检测** | 互联网和Telegram API连通性检查 | ✅ |
| **日志分析** | 自动分析错误和警告日志 | ✅ |
| **配置验证** | 环境变量和依赖包完整性检查 | ✅ |
| **故障恢复** | 自动重启、网络重连、错误处理 | ✅ |

### ⚙️ 运维管理功能

| 功能 | 描述 | 状态 |
|------|------|------|
| **统一管理界面** | 集成Bot和Guard的管理菜单 | ✅ |
| **自动构建** | Cargo自动依赖管理和编译 | ✅ |
| **自动更新** | 一键检查和更新到最新版本 | ✅ |
| **日志轮转** | 自动压缩和清理历史日志 | ✅ |
| **配置验证** | 启动前自动验证所有配置 | ✅ |
| **健康检查** | 实时系统健康状态检查 | ✅ |
| **备份恢复** | 自动备份重要配置和数据 | ✅ |
| **安全卸载** | 完整清理所有相关文件 | ✅ |

---

## 🎮 机器人命令

### 👤 用户命令

| 命令 | 功能 | 示例 |
|------|------|------|
| `/start` | 开始使用机器人 | `/start` |
| `/help` | 获取帮助信息 | `/help` |
| `机器码` | 直接发送机器码生成全版本激活码 | `发送你的机器码` |

### 👑 管理员命令

| 命令 | 功能 | 示例 |
|------|------|------|
| `/stats` | 查看使用统计 | `/stats` |
| `/users` | 查看用户列表 | `/users` |
| `/ban <用户ID>` | 拉黑用户 | `/ban 123456789` |
| `/unban <用户ID>` | 解除拉黑 | `/unban 123456789` |
| `/say <内容>` | 广播消息 | `/say 系统维护通知` |
| `/clear` | 清除统计数据 | `/clear` |
| `/cleanup` | 清理日志文件 | `/cleanup` |
| `/guard` | 获取最新自检报告 | `/guard` |

---

## 🔧 配置文件

### 📝 环境变量 (.env)

```env
# Telegram Bot 配置
BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
CHAT_ID=123456789

# 管理员ID列表 (用逗号分隔)
ADMIN_IDS=123456789,987654321

# 数据库配置
DATABASE_URL=sqlite:finalshell_bot.db

# 应用配置
MAX_USER_REQUESTS=3
LOG_LEVEL=info

# Guard守护进程配置 (秒)
GUARD_CHECK_INTERVAL=86400

# Rust 日志配置
RUST_LOG=finalunlock_rust=info,teloxide=info
```

### 

---

## 📊 性能对比

| 指标 | Python版本 | Rust版本 | 提升 |
|------|------------|----------|------|
| **内存占用** | ~150MB | ~45MB | 70%↓ |
| **启动时间** | ~3s | ~0.5s | 83%↓ |
| **并发处理** | ~100 req/s | ~1000 req/s | 900%↑ |
| **编译后大小** | N/A | ~15MB | - |
| **CPU使用率** | ~15% | ~5% | 67%↓ |

---

## 🔧 开发指南

### 🛠️ 本地开发

```bash
# 克隆项目
git clone https://github.com/xymn2023/finalunlock-rust.git
cd finalunlock-all

# 安装依赖
cargo build

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 运行开发版本
cargo run -- bot
```

### 📦 项目结构

```
finalunlock-rust/
├── src/
│   ├── main.rs          # 主入口
│   ├── config.rs        # 配置管理
│   ├── bot.rs          # Telegram机器人
│   ├── finalshell.rs   # 激活码生成
│   ├── guard.rs        # 守护进程
│   ├── database.rs     # 数据库操作
│   ├── models.rs       # 数据模型
│   └── utils.rs        # 工具函数
├── Cargo.toml          # 依赖配置
├── start.sh           # 启动脚本
├── env.example        # 环境变量示例
└── README-RUST.md     # 文档
```

---

## 🔍 故障排除

### 🚨 常见问题

#### 1. 编译失败

```bash
# 更新Rust工具链
rustup update

# 清理构建缓存
cargo clean

# 重新构建
cargo build --release
```

#### 2. 依赖问题

```bash
# 更新依赖
cargo update

# 检查依赖树
cargo tree
```

#### 3. 运行时错误

```bash
# 查看详细日志
RUST_LOG=debug ./target/release/finalunlock-rust bot

# 检查配置
./start.sh --status
```

---

## 🤝 贡献指南

### 🎯 如何贡献

1. **Fork项目**: 点击右上角Fork按钮
2. **创建分支**: `git checkout -b feature/your-feature`
3. **提交更改**: `git commit -am 'Add some feature'`
4. **推送分支**: `git push origin feature/your-feature`
5. **创建PR**: 在GitHub上创建Pull Request

### 🔍 代码规范

- 遵循Rust官方代码风格指南
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 确保所有测试通过 `cargo test`
- 添加适当的文档注释

---

## 📄 开源协议

本项目采用 [MIT License](LICENSE) 开源协议。

---

## 📞 联系方式

- **GitHub**: [xymn2023/finalunlock-rust](https://github.com/xymn2023/finalunlock-rust)
- **Issues**: [提交问题](https://github.com/xymn2023/finalunlock-rust/issues)
- **Demo Bot**: [@finalunlock-bot](https://t.me/toosvideo_bot)

---

<div align="center">
**🎉 感谢使用 finalunlock-rust！**

如果这个项目对您有帮助，请考虑给我们一个 ⭐ Star！

[⬆️ 回到顶部](#-finalshell-激活码-telegram-机器人-rust-版本)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=xymn2023/finalunlock-rust&type=Date)](https://www.star-history.com/#xymn2023/finalunlock-rust&Date)

</div>
