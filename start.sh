#!/bin/bash

# FinalShell 激活码机器人管理脚本
# Rust 版本
# 
# 🚀 新功能: 自动安装 Rust 环境
# - 🔍 自动检测 Rust 是否已安装
# - 📦 如果未安装，自动下载并安装 Rust
# - ⚙️ 自动配置环境变量

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 项目配置
PROJECT_NAME="finalunlock-all-rust"
SERVICE_NAME="finalshell-bot"
BINARY_NAME="finalunlock-all-rust"
PID_FILE="/tmp/${SERVICE_NAME}.pid"
GUARD_PID_FILE="/tmp/${SERVICE_NAME}-guard.pid"

# 自动加载 Rust 环境变量（如果存在）
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env" 2>/dev/null || true
fi

# 函数：打印彩色输出
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 函数：检查是否为root用户
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_warning "建议不要使用root用户运行此脚本"
        read -p "是否继续? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
}

# 函数：检查系统依赖
check_dependencies() {
    print_info "检查系统依赖..."
    
    # 检查Rust
    if ! command -v rustc &> /dev/null; then
        print_warning "Rust 未安装，开始自动安装..."
        print_info "正在下载并安装 Rust..."
        
        # 下载并安装 Rust
        if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
            print_success "Rust 安装成功"
            
            # 加载 Rust 环境变量
            export PATH="$HOME/.cargo/bin:$PATH"
            source "$HOME/.cargo/env" 2>/dev/null || true
            
            print_info "已自动加载 Rust 环境变量"
        else
            print_error "Rust 安装失败"
            print_info "请手动安装: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
            exit 1
        fi
    fi
    
    # 检查Cargo
    if ! command -v cargo &> /dev/null; then
        print_warning "Cargo 未找到，尝试重新加载环境变量..."
        
        # 尝试加载 Rust 环境变量
        export PATH="$HOME/.cargo/bin:$PATH"
        source "$HOME/.cargo/env" 2>/dev/null || true
        
        # 再次检查
        if ! command -v cargo &> /dev/null; then
            print_error "Cargo 仍然未找到，请检查 Rust 安装"
            print_info "手动加载环境: source ~/.cargo/env"
            exit 1
        else
            print_success "Cargo 已找到"
        fi
    fi
    
    print_success "系统依赖检查完成"
}

# 函数：构建项目
build_project() {
    print_info "构建项目..."
    
    if cargo build --release; then
        print_success "项目构建成功"
    else
        print_error "项目构建失败"
        exit 1
    fi
}

# 函数：初始化数据库
init_database() {
    print_info "初始化数据库..."
    
    # 检查数据库文件是否存在
    if [ -f "./data/finalshell_bot.db" ]; then
        print_success "数据库文件已存在，直接使用"
        return 0
    fi
    
    if [ -f "./finalshell_bot.db" ]; then
        print_success "数据库文件已存在，直接使用"
        return 0
    fi
    
    # 数据库文件不存在，需要创建
    print_info "数据库文件不存在，开始创建..."
    
    # 确保数据目录存在
    mkdir -p data
    chmod 775 data
    
    # 检查SQLite是否安装
    if ! command -v sqlite3 &> /dev/null; then
        print_warning "SQLite 未安装，尝试安装..."
        if command -v apt-get &> /dev/null; then
            apt-get update && apt-get install -y sqlite3
        elif command -v yum &> /dev/null; then
            yum install -y sqlite
        elif command -v apk &> /dev/null; then
            apk add sqlite
        else
            print_error "无法自动安装SQLite，请手动安装"
            exit 1
        fi
    fi
    
    # 使用SQL脚本初始化数据库
    if [ -f "init_db.sql" ]; then
        print_info "使用SQL脚本初始化数据库..."
        if sqlite3 ./data/finalshell_bot.db < init_db.sql; then
            print_success "数据库初始化成功"
            chmod 666 ./data/finalshell_bot.db
            # 复制到当前目录作为备用
            cp ./data/finalshell_bot.db ./finalshell_bot.db 2>/dev/null || true
        else
            print_error "SQL脚本执行失败"
            # 回退到应用程序初始化
            print_info "尝试使用应用程序初始化..."
            if ./target/release/${BINARY_NAME} init-db; then
                print_success "应用程序初始化数据库成功"
            else
                print_error "所有尝试都失败，请检查系统权限"
                exit 1
            fi
        fi
    else
        print_warning "SQL脚本不存在，使用应用程序初始化..."
        if ./target/release/${BINARY_NAME} init-db; then
            print_success "应用程序初始化数据库成功"
        else
            print_error "数据库初始化失败"
            exit 1
        fi
    fi
}

# 函数：检查配置文件
check_config() {
    if [[ ! -f ".env" ]]; then
        print_warning "配置文件 .env 不存在"
        if [[ -f "env.example" ]]; then
            print_info "复制示例配置文件..."
            cp env.example .env
            print_warning "请编辑 .env 文件配置你的 Bot Token 和 Chat ID"
            print_info "配置完成后请重新运行此脚本"
            exit 1
        else
            print_error "示例配置文件不存在"
            exit 1
        fi
    fi
    
    # 检查必要的环境变量
    source .env
    if [[ -z "$BOT_TOKEN" || -z "$CHAT_ID" ]]; then
        print_error "BOT_TOKEN 或 CHAT_ID 未配置"
        print_info "请编辑 .env 文件配置你的 Bot Token 和 Chat ID"
        exit 1
    fi
    
    print_success "配置检查完成"
}

# 函数：获取进程状态
get_process_status() {
    local pid_file=$1
    local service_name=$2
    
    if [[ -f "$pid_file" ]]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${GREEN}✅ 正在运行${NC} (PID: $pid)"
            return 0
        else
            rm -f "$pid_file"
            echo -e "${RED}❌ 未运行${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌ 未运行${NC}"
        return 1
    fi
}

# 函数：启动机器人
start_bot() {
    print_info "启动机器人..."
    
    if [[ -f "$PID_FILE" ]]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_warning "机器人已在运行 (PID: $pid)"
            return 0
        else
            rm -f "$PID_FILE"
        fi
    fi
    
    # 检查BOT_TOKEN是否有效
    source .env
    if [[ "$BOT_TOKEN" == "123456789:ABCdefGHIjklMNOpqrsTUVwxyz" ]]; then
        print_error "BOT_TOKEN 未配置，请在 .env 文件中设置有效的 Telegram Bot Token"
        print_info "请访问 https://t.me/BotFather 创建机器人并获取 Token"
        return 1
    fi
    
    # 启动机器人
    nohup ./target/release/${BINARY_NAME} bot > bot.log 2>&1 &
    local pid=$!
    echo $pid > "$PID_FILE"
    
    sleep 3
    
    if kill -0 "$pid" 2>/dev/null; then
        print_success "机器人启动成功 (PID: $pid)"
        print_info "查看日志: tail -f bot.log"
    else
        print_error "机器人启动失败"
        print_info "查看错误日志: cat bot.log"
        rm -f "$PID_FILE"
    fi
}

# 函数：停止机器人
stop_bot() {
    print_info "停止机器人..."
    
    if [[ -f "$PID_FILE" ]]; then
        local pid=$(cat "$PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            sleep 2
            
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid"
                sleep 1
            fi
            
            rm -f "$PID_FILE"
            print_success "机器人已停止"
        else
            rm -f "$PID_FILE"
            print_warning "机器人未在运行"
        fi
    else
        print_warning "机器人未在运行"
    fi
}

# 函数：启动Guard守护进程
start_guard() {
    print_info "启动Guard守护进程..."
    
    if [[ -f "$GUARD_PID_FILE" ]]; then
        local pid=$(cat "$GUARD_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            print_warning "Guard已在运行 (PID: $pid)"
            return 0
        else
            rm -f "$GUARD_PID_FILE"
        fi
    fi
    
    # 启动Guard
    nohup ./target/release/${BINARY_NAME} guard > guard.log 2>&1 &
    local pid=$!
    echo $pid > "$GUARD_PID_FILE"
    
    sleep 2
    
    if kill -0 "$pid" 2>/dev/null; then
        print_success "Guard启动成功 (PID: $pid)"
    else
        print_error "Guard启动失败"
        rm -f "$GUARD_PID_FILE"
        exit 1
    fi
}

# 函数：停止Guard守护进程
stop_guard() {
    print_info "停止Guard守护进程..."
    
    if [[ -f "$GUARD_PID_FILE" ]]; then
        local pid=$(cat "$GUARD_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid"
            sleep 2
            
            if kill -0 "$pid" 2>/dev/null; then
                kill -9 "$pid"
                sleep 1
            fi
            
            rm -f "$GUARD_PID_FILE"
            print_success "Guard已停止"
        else
            rm -f "$GUARD_PID_FILE"
            print_warning "Guard未在运行"
        fi
    else
        print_warning "Guard未在运行"
    fi
}

# 函数：重启服务
restart_services() {
    print_info "重启所有服务..."
    stop_guard
    stop_bot
    sleep 2
    start_bot
    start_guard
}

# 函数：查看日志
view_logs() {
    echo
    print_info "选择要查看的日志:"
    echo "1. 机器人日志 (bot.log)"
    echo "2. Guard日志 (guard.log)"
    echo "3. 实时机器人日志"
    echo "4. 实时Guard日志"
    echo "0. 返回主菜单"
    echo
    
    read -p "请选择 [0-4]: " choice
    
    case $choice in
        1)
            if [[ -f "bot.log" ]]; then
                less bot.log
            else
                print_warning "bot.log 文件不存在"
            fi
            ;;
        2)
            if [[ -f "guard.log" ]]; then
                less guard.log
            else
                print_warning "guard.log 文件不存在"
            fi
            ;;
        3)
            if [[ -f "bot.log" ]]; then
                tail -f bot.log
            else
                print_warning "bot.log 文件不存在"
            fi
            ;;
        4)
            if [[ -f "guard.log" ]]; then
                tail -f guard.log
            else
                print_warning "guard.log 文件不存在"
            fi
            ;;
        0)
            return
            ;;
        *)
            print_error "无效选择"
            ;;
    esac
}

# 函数：显示系统状态
show_status() {
    echo
    print_info "系统状态信息:"
    echo "================================"
    echo -e "${CYAN}项目名称:${NC} $PROJECT_NAME"
    echo -e "${CYAN}工作目录:${NC} $(pwd)"
    echo
    echo -e "${CYAN}机器人状态:${NC} $(get_process_status "$PID_FILE" "Bot")"
    echo -e "${CYAN}Guard状态:${NC} $(get_process_status "$GUARD_PID_FILE" "Guard")"
    echo
    
    # 显示系统资源使用
    if command -v free &> /dev/null; then
        echo -e "${CYAN}内存使用:${NC}"
        free -h | grep -E "(Mem|内存)"
    fi
    
    if command -v df &> /dev/null; then
        echo -e "${CYAN}磁盘使用:${NC}"
        df -h . | tail -1
    fi
    
    echo "================================"
    echo
}

# 函数：清理日志
cleanup_logs() {
    print_info "清理日志文件..."
    
    find . -name "*.log" -type f -mtime +7 -exec rm -f {} \;
    
    print_success "日志清理完成"
}

# 函数：检查更新
check_updates() {
    print_info "检查项目更新..."
    
    if command -v git &> /dev/null; then
        if git fetch && git status | grep -q "behind"; then
            print_info "发现新版本!"
            read -p "是否更新到最新版本? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                git pull
                build_project
                restart_services
                print_success "更新完成"
            fi
        else
            print_success "已是最新版本"
        fi
    else
        print_warning "Git未安装，无法检查更新"
    fi
}

# 函数：手动执行健康检查
manual_health_check() {
    print_info "执行手动健康检查..."
    
    if ./target/release/${BINARY_NAME} check; then
        print_success "健康检查完成"
    else
        print_error "健康检查失败"
    fi
}

# 函数：显示帮助信息
show_help() {
    echo
    echo -e "${BLUE}FinalShell 激活码机器人管理脚本${NC}"
    echo "================================"
    echo
    echo "使用方法:"
    echo "  ./start.sh [选项]"
    echo
    echo "选项:"
    echo "  -h, --help     显示帮助信息"
    echo "  --start        启动所有服务"
    echo "  --stop         停止所有服务"
    echo "  --restart      重启所有服务"
    echo "  --status       显示状态信息"
    echo "  --build        仅构建项目"
    echo "  --check        执行健康检查"
    echo
    echo "交互模式:"
    echo "  ./start.sh     进入交互式管理界面"
    echo
}

# 函数：显示主菜单
show_main_menu() {
    clear
    echo -e "${BLUE}================================${NC}"
    echo -e "${BLUE}FinalShell 机器人管理菜单${NC}"
    echo -e "${BLUE}================================${NC}"
    echo
    
    # 显示当前状态
    echo -e "${CYAN}当前状态:${NC}"
    echo -e "机器人: $(get_process_status "$PID_FILE" "Bot")"
    echo -e "Guard: $(get_process_status "$GUARD_PID_FILE" "Guard")"
    echo
    
    echo -e "${PURPLE}=== 🤖 服务管理 ===${NC}"
    echo "[1] 启动所有服务"
    echo "[2] 停止所有服务"
    echo "[3] 重启所有服务"
    echo "[4] 启动机器人"
    echo "[5] 停止机器人"
    echo "[6] 启动Guard"
    echo "[7] 停止Guard"
    echo
    echo -e "${PURPLE}=== 📋 系统监控 ===${NC}"
    echo "[8] 查看系统状态"
    echo "[9] 查看日志"
    echo "[10] 手动健康检查"
    echo
    echo -e "${PURPLE}=== 🔧 维护功能 ===${NC}"
    echo "[11] 重新构建项目"
    echo "[12] 初始化数据库"
    echo "[13] 清理日志"
    echo "[14] 检查更新"
    echo
    echo "[0] 退出"
    echo
}

# 主函数
main() {
    # 处理命令行参数
    case "${1:-}" in
        -h|--help)
            show_help
            exit 0
            ;;
        --start)
            check_dependencies
            check_config
            build_project
            init_database
            start_bot
            start_guard
            exit 0
            ;;
        --stop)
            stop_guard
            stop_bot
            exit 0
            ;;
        --restart)
            restart_services
            exit 0
            ;;
        --status)
            show_status
            exit 0
            ;;
        --build)
            check_dependencies
            build_project
            exit 0
            ;;
        --check)
            manual_health_check
            exit 0
            ;;
    esac
    
    # 初始检查
    check_root
    check_dependencies
    check_config
    
    # 如果二进制文件不存在，先构建项目
    if [[ ! -f "target/release/${BINARY_NAME}" ]]; then
        print_info "二进制文件不存在，正在构建项目..."
        build_project
        init_database
    fi
    
    # 进入交互模式
    while true; do
        show_main_menu
        read -p "请选择操作 [0-14]: " choice
        
        case $choice in
            1)
                start_bot
                start_guard
                ;;
            2)
                stop_guard
                stop_bot
                ;;
            3)
                restart_services
                ;;
            4)
                start_bot
                ;;
            5)
                stop_bot
                ;;
            6)
                start_guard
                ;;
            7)
                stop_guard
                ;;
            8)
                show_status
                ;;
            9)
                view_logs
                ;;
            10)
                manual_health_check
                ;;
            11)
                build_project
                ;;
            12)
                init_database
                ;;
            13)
                cleanup_logs
                ;;
            14)
                check_updates
                ;;
            0)
                print_info "再见!"
                exit 0
                ;;
            *)
                print_error "无效选择，请重试"
                ;;
        esac
        
        echo
        read -p "按回车键继续..." -r
    done
}

# 脚本入口
main "$@"
