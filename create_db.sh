#!/bin/bash

# FinalShell 激活码机器人数据库创建脚本
# 提前创建数据库文件，运行时直接读取

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# 检查SQLite是否安装
check_sqlite() {
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
    print_success "SQLite 已安装"
}

# 创建数据库目录
create_directories() {
    print_info "创建数据库目录..."
    mkdir -p data
    chmod 775 data
    print_success "数据库目录创建成功"
}

# 初始化数据库
init_database() {
    local db_path="${1:-./finalshell_bot.db}"
    
    print_info "初始化数据库: $db_path"
    
    # 执行SQL脚本
    if sqlite3 "$db_path" < init_db.sql; then
        print_success "数据库初始化成功"
        
        # 设置文件权限
        chmod 666 "$db_path"
        print_info "数据库文件权限设置成功"
        
        # 验证数据库
        verify_database "$db_path"
    else
        print_error "数据库初始化失败"
        exit 1
    fi
}

# 验证数据库
verify_database() {
    local db_path="$1"
    
    print_info "验证数据库结构..."
    
    # 检查用户表
    if sqlite3 "$db_path" "SELECT name FROM sqlite_master WHERE type='table' AND name='users';" | grep -q "users"; then
        print_success "用户表存在"
    else
        print_error "用户表不存在"
        exit 1
    fi
    
    # 检查激活日志表
    if sqlite3 "$db_path" "SELECT name FROM sqlite_master WHERE type='table' AND name='activation_logs';" | grep -q "activation_logs"; then
        print_success "激活日志表存在"
    else
        print_error "激活日志表不存在"
        exit 1
    fi
    
    # 检查系统统计表
    if sqlite3 "$db_path" "SELECT name FROM sqlite_master WHERE type='table' AND name='system_stats';" | grep -q "system_stats"; then
        print_success "系统统计表存在"
    else
        print_error "系统统计表不存在"
        exit 1
    fi
    
    # 检查初始数据
    local count=$(sqlite3 "$db_path" "SELECT COUNT(*) FROM system_stats;")
    if [ "$count" -ge 1 ]; then
        print_success "初始数据存在"
    else
        print_error "初始数据不存在"
        exit 1
    fi
    
    print_success "数据库验证成功"
}

# 主函数
main() {
    print_info "=== FinalShell 数据库创建脚本 ==="
    
    # 检查SQLite
    check_sqlite
    
    # 创建目录
    create_directories
    
    # 初始化数据库
    init_database "./data/finalshell_bot.db"
    
    # 复制到当前目录作为备用
    cp "./data/finalshell_bot.db" "./finalshell_bot.db" 2>/dev/null || true
    
    print_info ""
    print_success "数据库创建完成！"
    print_info "数据库文件位置:"
    print_info "1. ./data/finalshell_bot.db"
    print_info "2. ./finalshell_bot.db (备用)"
    print_info ""
    print_info "请确保在 .env 文件中配置正确的数据库路径:"
    print_info "DATABASE_URL=sqlite:./data/finalshell_bot.db"
}

# 脚本入口
main "$@"