#!/bin/bash
# UMSKZ 特殊总账标识 - 快速验证脚本
# 用途: 快速验证功能是否正常工作

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 配置
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"
DB_NAME="${DB_NAME:-gl_service}"
DB_USER="${DB_USER:-postgres}"

# 打印函数
print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_test() {
    echo -e "${YELLOW}▶ $1${NC}"
}

print_pass() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_fail() {
    echo -e "${RED}✗ $1${NC}"
}

# 测试计数器
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# 运行测试
run_test() {
    local test_name="$1"
    local test_command="$2"

    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    print_test "$test_name"

    if eval "$test_command" > /dev/null 2>&1; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        print_pass "$test_name"
        return 0
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        print_fail "$test_name"
        return 1
    fi
}

# 主函数
main() {
    print_header "UMSKZ 功能验证"

    # 1. 数据库连接测试
    print_header "1. 数据库连接测试"
    run_test "数据库连接" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c 'SELECT 1;'"

    # 2. 字段验证
    print_header "2. 字段验证"
    run_test "special_gl_indicator 字段存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'journal_entry_lines' AND column_name = 'special_gl_indicator';\" | grep -q '1'"

    # 3. 约束验证
    print_header "3. 约束验证"
    run_test "chk_special_gl_indicator 约束存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.table_constraints WHERE table_name = 'journal_entry_lines' AND constraint_name = 'chk_special_gl_indicator';\" | grep -q '1'"

    # 4. 索引验证
    print_header "4. 索引验证"
    run_test "特殊总账索引存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM pg_indexes WHERE tablename = 'journal_entry_lines' AND indexname LIKE '%special_gl%';\" | grep -q '[2-9]'"

    # 5. 视图验证
    print_header "5. 视图验证"
    run_test "v_special_gl_items 视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.views WHERE table_name = 'v_special_gl_items';\" | grep -q '1'"

    run_test "v_special_gl_summary 视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.views WHERE table_name = 'v_special_gl_summary';\" | grep -q '1'"

    run_test "v_down_payment_balance 视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.views WHERE table_name = 'v_down_payment_balance';\" | grep -q '1'"

    run_test "v_advance_payment_balance 视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.views WHERE table_name = 'v_advance_payment_balance';\" | grep -q '1'"

    run_test "v_bill_maturity_analysis 视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.views WHERE table_name = 'v_bill_maturity_analysis';\" | grep -q '1'"

    run_test "v_special_gl_risk_alert 视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.views WHERE table_name = 'v_special_gl_risk_alert';\" | grep -q '1'"

    # 6. 物化视图验证
    print_header "6. 物化视图验证"
    run_test "mv_special_gl_balance 物化视图存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM pg_matviews WHERE matviewname = 'mv_special_gl_balance';\" | grep -q '1'"

    # 7. 函数验证
    print_header "7. 函数验证"
    run_test "refresh_special_gl_materialized_views 函数存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.routines WHERE routine_name = 'refresh_special_gl_materialized_views';\" | grep -q '1'"

    run_test "analyze_special_gl_tables 函数存在" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -t -c \"SELECT COUNT(*) FROM information_schema.routines WHERE routine_name = 'analyze_special_gl_tables';\" | grep -q '1'"

    # 8. 功能测试
    print_header "8. 功能测试"
    run_test "视图可查询" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c 'SELECT COUNT(*) FROM v_special_gl_items;'"

    run_test "物化视图可查询" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c 'SELECT COUNT(*) FROM mv_special_gl_balance;'"

    run_test "刷新物化视图函数可执行" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c 'SELECT refresh_special_gl_materialized_views();'"

    run_test "统计信息收集函数可执行" \
        "psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -c 'SELECT analyze_special_gl_tables();'"

    # 9. 代码编译测试
    print_header "9. 代码编译测试"
    if command -v cargo &> /dev/null; then
        run_test "GL Service 编译" \
            "cargo check --package gl-service"

        run_test "AP Service 编译" \
            "cargo check --package ap-service"

        run_test "AR Service 编译" \
            "cargo check --package ar-service"

        run_test "Cuba Finance 编译" \
            "cargo check --package cuba-finance"
    else
        print_test "跳过代码编译测试 (cargo 未安装)"
    fi

    # 10. 单元测试
    print_header "10. 单元测试"
    if command -v cargo &> /dev/null; then
        run_test "Domain Model 测试" \
            "cargo test --package gl-service --lib domain::aggregates::journal_entry::tests"
    else
        print_test "跳过单元测试 (cargo 未安装)"
    fi

    # 显示结果
    print_header "测试结果"
    echo "总测试数: $TOTAL_TESTS"
    echo -e "${GREEN}通过: $PASSED_TESTS${NC}"
    echo -e "${RED}失败: $FAILED_TESTS${NC}"

    if [ $FAILED_TESTS -eq 0 ]; then
        echo ""
        echo -e "${GREEN}========================================${NC}"
        echo -e "${GREEN}  ✓ 所有测试通过！${NC}"
        echo -e "${GREEN}========================================${NC}"
        echo ""
        return 0
    else
        echo ""
        echo -e "${RED}========================================${NC}"
        echo -e "${RED}  ✗ 有 $FAILED_TESTS 个测试失败${NC}"
        echo -e "${RED}========================================${NC}"
        echo ""
        return 1
    fi
}

# 执行主函数
main "$@"
