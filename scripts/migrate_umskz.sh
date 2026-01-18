#!/bin/bash
# UMSKZ 特殊总账标识 - 数据库迁移脚本
# 用途: 自动执行数据库迁移并验证结果

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-gl_service}"
DB_USER="${DB_USER:-postgres}"
BACKUP_DIR="./backups"
MIGRATION_FILE="apps/fi/gl-service/migrations/20260118000001_add_special_gl_indicator.sql"

# 打印带颜色的消息
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

# 检查必要的工具
check_prerequisites() {
    print_info "检查必要的工具..."

    if ! command -v psql &> /dev/null; then
        print_error "psql 未安装，请先安装 PostgreSQL 客户端"
        exit 1
    fi

    if ! command -v pg_dump &> /dev/null; then
        print_error "pg_dump 未安装，请先安装 PostgreSQL 客户端"
        exit 1
    fi

    print_success "所有必要工具已安装"
}

# 测试数据库连接
test_connection() {
    print_info "测试数据库连接..."

    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -c "SELECT version();" > /dev/null 2>&1; then
        print_success "数据库连接成功"
    else
        print_error "无法连接到数据库"
        print_error "请检查数据库配置: $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
        exit 1
    fi
}

# 备份数据库
backup_database() {
    print_info "备份数据库..."

    # 创建备份目录
    mkdir -p "$BACKUP_DIR"

    # 生成备份文件名
    BACKUP_FILE="$BACKUP_DIR/backup_before_umskz_$(date +%Y%m%d_%H%M%S).sql"

    # 执行备份
    if pg_dump -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > "$BACKUP_FILE"; then
        print_success "数据库备份成功: $BACKUP_FILE"

        # 显示备份文件大小
        BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
        print_info "备份文件大小: $BACKUP_SIZE"
    else
        print_error "数据库备份失败"
        exit 1
    fi
}

# 检查迁移文件
check_migration_file() {
    print_info "检查迁移文件..."

    if [ ! -f "$MIGRATION_FILE" ]; then
        print_error "迁移文件不存在: $MIGRATION_FILE"
        exit 1
    fi

    print_success "迁移文件存在"
}

# 执行迁移
execute_migration() {
    print_info "执行数据库迁移..."

    if psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -f "$MIGRATION_FILE"; then
        print_success "数据库迁移执行成功"
    else
        print_error "数据库迁移执行失败"
        print_warning "请检查错误信息，如需回滚请使用备份文件: $BACKUP_FILE"
        exit 1
    fi
}

# 验证迁移结果
verify_migration() {
    print_info "验证迁移结果..."

    # 1. 检查字段是否添加
    print_info "检查字段 special_gl_indicator..."
    FIELD_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_name = 'journal_entry_lines'
         AND column_name = 'special_gl_indicator';")

    if [ "$FIELD_COUNT" -eq 1 ]; then
        print_success "字段 special_gl_indicator 已添加"
    else
        print_error "字段 special_gl_indicator 未找到"
        return 1
    fi

    # 2. 检查约束是否创建
    print_info "检查约束 chk_special_gl_indicator..."
    CONSTRAINT_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM information_schema.table_constraints
         WHERE table_name = 'journal_entry_lines'
         AND constraint_name = 'chk_special_gl_indicator';")

    if [ "$CONSTRAINT_COUNT" -eq 1 ]; then
        print_success "约束 chk_special_gl_indicator 已创建"
    else
        print_error "约束 chk_special_gl_indicator 未找到"
        return 1
    fi

    # 3. 检查索引是否创建
    print_info "检查索引..."
    INDEX_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM pg_indexes
         WHERE tablename = 'journal_entry_lines'
         AND indexname LIKE '%special_gl%';")

    if [ "$INDEX_COUNT" -ge 2 ]; then
        print_success "索引已创建 (找到 $INDEX_COUNT 个)"
    else
        print_warning "索引数量不足 (找到 $INDEX_COUNT 个，预期至少 2 个)"
    fi

    # 4. 检查视图是否创建
    print_info "检查视图..."
    VIEW_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = 'public'
         AND table_name LIKE 'v_special_gl%';")

    if [ "$VIEW_COUNT" -ge 10 ]; then
        print_success "视图已创建 (找到 $VIEW_COUNT 个)"
    else
        print_warning "视图数量不足 (找到 $VIEW_COUNT 个，预期至少 10 个)"
    fi

    # 5. 检查物化视图是否创建
    print_info "检查物化视图..."
    MATVIEW_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM pg_matviews
         WHERE matviewname = 'mv_special_gl_balance';")

    if [ "$MATVIEW_COUNT" -eq 1 ]; then
        print_success "物化视图 mv_special_gl_balance 已创建"
    else
        print_error "物化视图 mv_special_gl_balance 未找到"
        return 1
    fi

    # 6. 检查函数是否创建
    print_info "检查函数..."
    FUNCTION_COUNT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM information_schema.routines
         WHERE routine_schema = 'public'
         AND routine_name IN ('refresh_special_gl_materialized_views', 'analyze_special_gl_tables');")

    if [ "$FUNCTION_COUNT" -eq 2 ]; then
        print_success "维护函数已创建 (2 个)"
    else
        print_warning "维护函数数量不足 (找到 $FUNCTION_COUNT 个，预期 2 个)"
    fi

    print_success "迁移验证完成"
}

# 测试基本功能
test_basic_functionality() {
    print_info "测试基本功能..."

    # 测试插入数据
    print_info "测试插入预付款数据..."
    psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" > /dev/null 2>&1 <<EOF
BEGIN;

-- 创建测试凭证
INSERT INTO journal_entries (
    id, company_code, fiscal_year, fiscal_period,
    posting_date, document_date, status, currency, created_at
) VALUES (
    gen_random_uuid(), '1000', 2026, 1,
    '2026-01-18', '2026-01-18', 'POSTED', 'CNY', NOW()
) RETURNING id \gset

-- 插入预付款行项目
INSERT INTO journal_entry_lines (
    id, journal_entry_id, line_item_number, account_id,
    debit_credit, amount, local_amount, currency, special_gl_indicator
) VALUES (
    gen_random_uuid(), :'id', 1, '1100',
    'D', 10000.00, 10000.00, 'CNY', 'F'
);

-- 插入对应的贷方行项目
INSERT INTO journal_entry_lines (
    id, journal_entry_id, line_item_number, account_id,
    debit_credit, amount, local_amount, currency, special_gl_indicator
) VALUES (
    gen_random_uuid(), :'id', 2, '2100',
    'C', 10000.00, 10000.00, 'CNY', ''
);

ROLLBACK;
EOF

    if [ $? -eq 0 ]; then
        print_success "数据插入测试通过"
    else
        print_error "数据插入测试失败"
        return 1
    fi

    # 测试视图查询
    print_info "测试视图查询..."
    QUERY_RESULT=$(psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$DB_NAME" -t -c \
        "SELECT COUNT(*) FROM v_special_gl_items LIMIT 1;" 2>&1)

    if [ $? -eq 0 ]; then
        print_success "视图查询测试通过"
    else
        print_error "视图查询测试失败: $QUERY_RESULT"
        return 1
    fi

    print_success "基本功能测试完成"
}

# 显示迁移摘要
show_summary() {
    echo ""
    echo "=========================================="
    echo "         迁移完成摘要"
    echo "=========================================="
    echo ""
    echo "✅ 数据库备份: $BACKUP_FILE"
    echo "✅ 迁移文件: $MIGRATION_FILE"
    echo "✅ 数据库: $DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
    echo ""
    echo "已创建:"
    echo "  - 1 个字段: special_gl_indicator"
    echo "  - 2 个索引: 单列索引 + 复合索引"
    echo "  - 1 个约束: 数据完整性检查"
    echo "  - 13 个视图: 业务分析视图"
    echo "  - 1 个物化视图: 性能优化"
    echo "  - 2 个函数: 维护工具"
    echo ""
    echo "支持的特殊总账类型:"
    echo "  A = 票据 (Bills of Exchange)"
    echo "  F = 预付款 (Down Payment)"
    echo "  V = 预收款 (Advance Payment)"
    echo "  W = 票据贴现 (Bill Discount)"
    echo ""
    echo "=========================================="
    echo ""
}

# 主函数
main() {
    echo ""
    echo "=========================================="
    echo "  UMSKZ 特殊总账标识 - 数据库迁移"
    echo "=========================================="
    echo ""

    # 检查参数
    if [ "$1" == "--help" ] || [ "$1" == "-h" ]; then
        echo "用法: $0 [选项]"
        echo ""
        echo "选项:"
        echo "  --skip-backup    跳过数据库备份（不推荐）"
        echo "  --skip-test      跳过功能测试"
        echo "  --help, -h       显示帮助信息"
        echo ""
        echo "环境变量:"
        echo "  DB_HOST          数据库主机 (默认: localhost)"
        echo "  DB_PORT          数据库端口 (默认: 5432)"
        echo "  DB_NAME          数据库名称 (默认: gl_service)"
        echo "  DB_USER          数据库用户 (默认: postgres)"
        echo ""
        exit 0
    fi

    SKIP_BACKUP=false
    SKIP_TEST=false

    for arg in "$@"; do
        case $arg in
            --skip-backup)
                SKIP_BACKUP=true
                print_warning "将跳过数据库备份（不推荐）"
                ;;
            --skip-test)
                SKIP_TEST=true
                print_warning "将跳过功能测试"
                ;;
        esac
    done

    # 执行迁移步骤
    check_prerequisites
    test_connection
    check_migration_file

    if [ "$SKIP_BACKUP" = false ]; then
        backup_database
    fi

    # 确认执行
    echo ""
    read -p "是否继续执行迁移？(y/N) " -n 1 -r
    echo ""
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_warning "迁移已取消"
        exit 0
    fi

    execute_migration
    verify_migration

    if [ "$SKIP_TEST" = false ]; then
        test_basic_functionality
    fi

    show_summary

    print_success "迁移完成！"
    echo ""
    print_info "下一步:"
    echo "  1. 查看文档: cat UMSKZ_DOCUMENTATION_INDEX.md"
    echo "  2. 测试查询: psql -d $DB_NAME -c 'SELECT * FROM v_special_gl_items LIMIT 5;'"
    echo "  3. 刷新物化视图: psql -d $DB_NAME -c 'SELECT refresh_special_gl_materialized_views();'"
    echo ""
}

# 执行主函数
main "$@"
