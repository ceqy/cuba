-- ============================================================================
-- COA Service Database Schema
-- 会计科目表（Chart of Accounts）管理服务数据库结构
-- 设计基线: SAP S/4HANA SKA1/SKB1/SKAT 表结构
-- ============================================================================

-- ----------------------------------------------------------------------------
-- 科目表主表 (Chart of Accounts Master)
-- 对应 SAP T004 表
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS chart_of_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chart_code VARCHAR(4) NOT NULL UNIQUE,           -- 科目表代码 (KTOPL)
    chart_name VARCHAR(100) NOT NULL,                -- 科目表名称
    description TEXT,                                 -- 描述

    -- 科目表类型
    accounting_standard VARCHAR(20) NOT NULL,         -- 会计准则 (CN_GAAP/IFRS/US_GAAP)
    country_code VARCHAR(2),                          -- 国家代码
    language VARCHAR(2) NOT NULL DEFAULT 'ZH',        -- 默认语言

    -- 编码规则
    account_length INT NOT NULL DEFAULT 10,           -- 科目代码长度
    segment_structure VARCHAR(50),                    -- 分段结构 (如: 4-2-2-2)

    -- 状态
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(50),
    changed_at TIMESTAMPTZ,

    -- 多租户
    tenant_id VARCHAR(50)
);

-- ----------------------------------------------------------------------------
-- 科目主数据表 (GL Account Master)
-- 对应 SAP SKA1 表
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS gl_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chart_code VARCHAR(4) NOT NULL,                   -- 科目表代码 (KTOPL)
    account_code VARCHAR(10) NOT NULL,                -- 科目代码 (SAKNR) - 10位数字

    -- 基础信息
    account_name VARCHAR(100) NOT NULL,               -- 科目名称 (TXT50)
    account_name_long VARCHAR(200),                   -- 科目长名称
    search_key VARCHAR(50),                           -- 搜索关键词

    -- 科目分类
    account_nature VARCHAR(20) NOT NULL,              -- 科目性质 (ASSET/LIABILITY/EQUITY/REVENUE/EXPENSE)
    account_category VARCHAR(20) NOT NULL,            -- 科目分类 (BALANCE_SHEET/INCOME_STATEMENT)
    account_group VARCHAR(10),                        -- 科目组 (KTOKS)

    -- 层级信息
    account_level INT NOT NULL DEFAULT 1,             -- 科目级次 (1-5)
    parent_account VARCHAR(10),                       -- 父科目代码
    is_leaf_account BOOLEAN NOT NULL DEFAULT TRUE,    -- 是否末级科目
    full_path VARCHAR(100),                           -- 完整路径 (如: 1000/1001/100101)
    depth INT DEFAULT 0,                              -- 深度（从根节点开始）

    -- 控制信息
    is_postable BOOLEAN NOT NULL DEFAULT TRUE,        -- 是否可过账
    is_cost_element BOOLEAN DEFAULT FALSE,            -- 是否成本要素
    line_item_display BOOLEAN DEFAULT TRUE,           -- 行项目显示 (XOPVW)
    open_item_management BOOLEAN DEFAULT FALSE,       -- 未清项管理 (XKRES)
    balance_indicator CHAR(1) DEFAULT 'D',            -- 余额方向 (D=Debit/C=Credit)

    -- 财务属性
    currency VARCHAR(3),                              -- 科目币种（NULL=使用公司代码币种）
    only_local_currency BOOLEAN DEFAULT TRUE,         -- 仅本币过账 (XGKON)
    exchange_rate_diff BOOLEAN DEFAULT FALSE,         -- 汇兑差异科目

    -- 税务
    tax_relevant BOOLEAN DEFAULT FALSE,               -- 税务相关科目
    tax_category VARCHAR(10),                         -- 税种

    -- 状态与有效性
    status VARCHAR(20) NOT NULL DEFAULT 'ACTIVE',     -- 科目状态 (ACTIVE/INACTIVE/BLOCKED/MARKED_FOR_DELETION)
    valid_from DATE,                                  -- 有效起始日期
    valid_to DATE,                                    -- 有效结束日期

    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(50),
    changed_at TIMESTAMPTZ,

    -- 多租户
    tenant_id VARCHAR(50),

    -- 约束
    CONSTRAINT uq_gl_account UNIQUE (chart_code, account_code),
    CONSTRAINT fk_chart_code FOREIGN KEY (chart_code) REFERENCES chart_of_accounts(chart_code),
    CONSTRAINT chk_account_code_format CHECK (account_code ~ '^[0-9]{1,10}$'),  -- 仅允许数字
    CONSTRAINT chk_balance_indicator CHECK (balance_indicator IN ('D', 'C')),
    CONSTRAINT chk_status CHECK (status IN ('ACTIVE', 'INACTIVE', 'BLOCKED', 'MARKED_FOR_DELETION'))
);

-- ----------------------------------------------------------------------------
-- 公司代码级科目数据表 (Company Code Account Data)
-- 对应 SAP SKB1 表
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS company_code_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_code VARCHAR(4) NOT NULL,                 -- 公司代码 (BUKRS)
    account_code VARCHAR(10) NOT NULL,                -- 科目代码 (SAKNR)
    chart_code VARCHAR(4) NOT NULL,                   -- 科目表代码

    -- 公司代码级控制
    posting_blocked BOOLEAN DEFAULT FALSE,            -- 过账冻结 (XSPEB)
    reconciliation_account_type VARCHAR(1),           -- 统驭科目类型 (D=Customer/K=Supplier/S=GL)
    field_status_group VARCHAR(4),                    -- 字段状态组 (FSTAG)

    -- 自动过账
    automatic_postings BOOLEAN DEFAULT FALSE,         -- 自动过账标识
    sort_key VARCHAR(3),                              -- 排序码 (ZUAWA)

    -- 税务
    tax_code VARCHAR(2),                              -- 默认税码

    -- 现金流量
    cash_flow_category VARCHAR(10),                   -- 现金流量分类

    -- 统计
    planning_group VARCHAR(10),                       -- 计划组
    tolerance_group VARCHAR(4),                       -- 容差组

    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(50),
    changed_at TIMESTAMPTZ,

    -- 多租户
    tenant_id VARCHAR(50),

    -- 约束
    CONSTRAINT uq_company_account UNIQUE (company_code, account_code),
    CONSTRAINT fk_gl_account FOREIGN KEY (chart_code, account_code)
        REFERENCES gl_accounts(chart_code, account_code) ON DELETE CASCADE,
    CONSTRAINT chk_recon_type CHECK (reconciliation_account_type IN ('D', 'K', 'S', NULL))
);

-- ----------------------------------------------------------------------------
-- 科目文本表（多语言支持）
-- 对应 SAP SKAT 表
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS account_texts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chart_code VARCHAR(4) NOT NULL,                   -- 科目表代码
    account_code VARCHAR(10) NOT NULL,                -- 科目代码
    language_code VARCHAR(2) NOT NULL,                -- 语言代码 (EN/ZH/DE/FR)

    -- 文本内容
    short_text VARCHAR(20),                           -- 短文本 (20字符)
    medium_text VARCHAR(50),                          -- 中文本 (50字符)
    long_text VARCHAR(200),                           -- 长文本 (200字符)
    description TEXT,                                 -- 详细描述

    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(50),
    changed_at TIMESTAMPTZ,

    -- 约束
    CONSTRAINT uq_account_text UNIQUE (chart_code, account_code, language_code),
    CONSTRAINT fk_account_text FOREIGN KEY (chart_code, account_code)
        REFERENCES gl_accounts(chart_code, account_code) ON DELETE CASCADE
);

-- ----------------------------------------------------------------------------
-- 科目组表 (Account Group)
-- 对应 SAP T004K 表
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS account_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    chart_code VARCHAR(4) NOT NULL,                   -- 科目表代码
    group_code VARCHAR(10) NOT NULL,                  -- 科目组代码

    -- 基础信息
    group_name VARCHAR(100) NOT NULL,                 -- 科目组名称
    description TEXT,                                 -- 描述

    -- 科目性质
    account_nature VARCHAR(20),                       -- 科目性质

    -- 编号范围
    number_range_from VARCHAR(10),                    -- 编号范围起始
    number_range_to VARCHAR(10),                      -- 编号范围结束

    -- 字段状态控制
    field_status_variant VARCHAR(4),                  -- 字段状态变式

    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(50),
    changed_at TIMESTAMPTZ,

    -- 多租户
    tenant_id VARCHAR(50),

    -- 约束
    CONSTRAINT uq_account_group UNIQUE (chart_code, group_code),
    CONSTRAINT fk_group_chart FOREIGN KEY (chart_code) REFERENCES chart_of_accounts(chart_code)
);

-- ----------------------------------------------------------------------------
-- 科目验证规则表 (Account Validation Rules)
-- 用于定义科目的验证规则
-- ----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS account_validation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_code VARCHAR(20) NOT NULL UNIQUE,            -- 规则代码
    rule_name VARCHAR(100) NOT NULL,                  -- 规则名称
    rule_type VARCHAR(20) NOT NULL,                   -- 规则类型 (MANDATORY_FIELD/VALUE_CHECK/BALANCE_CHECK)

    -- 规则定义
    target_field VARCHAR(50),                         -- 目标字段
    validation_logic TEXT,                            -- 验证逻辑（JSON 格式）
    error_message VARCHAR(200),                       -- 错误消息

    -- 应用范围
    chart_code VARCHAR(4),                            -- 科目表代码（NULL=全部）
    account_nature VARCHAR(20),                       -- 科目性质（NULL=全部）

    -- 状态
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    -- 审计字段
    created_by VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    changed_by VARCHAR(50),
    changed_at TIMESTAMPTZ
);

-- ============================================================================
-- 索引 (Indexes)
-- ============================================================================

-- 科目表索引
CREATE INDEX IF NOT EXISTS idx_coa_standard ON chart_of_accounts(accounting_standard);
CREATE INDEX IF NOT EXISTS idx_coa_active ON chart_of_accounts(is_active);

-- 科目主数据索引
CREATE INDEX IF NOT EXISTS idx_gl_accounts_chart ON gl_accounts(chart_code);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_code ON gl_accounts(account_code);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_nature ON gl_accounts(account_nature);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_category ON gl_accounts(account_category);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_parent ON gl_accounts(parent_account);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_status ON gl_accounts(status);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_postable ON gl_accounts(is_postable) WHERE is_postable = TRUE;
CREATE INDEX IF NOT EXISTS idx_gl_accounts_level ON gl_accounts(account_level);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_group ON gl_accounts(account_group);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_path ON gl_accounts USING GIN (to_tsvector('simple', full_path));
CREATE INDEX IF NOT EXISTS idx_gl_accounts_name ON gl_accounts USING GIN (to_tsvector('simple', account_name));

-- 公司代码级索引
CREATE INDEX IF NOT EXISTS idx_company_accounts_company ON company_code_accounts(company_code);
CREATE INDEX IF NOT EXISTS idx_company_accounts_account ON company_code_accounts(account_code);
CREATE INDEX IF NOT EXISTS idx_company_accounts_recon ON company_code_accounts(reconciliation_account_type);

-- 科目文本索引
CREATE INDEX IF NOT EXISTS idx_account_texts_lang ON account_texts(language_code);
CREATE INDEX IF NOT EXISTS idx_account_texts_account ON account_texts(chart_code, account_code);

-- 科目组索引
CREATE INDEX IF NOT EXISTS idx_account_groups_chart ON account_groups(chart_code);
CREATE INDEX IF NOT EXISTS idx_account_groups_nature ON account_groups(account_nature);

-- ============================================================================
-- 视图 (Views)
-- ============================================================================

-- 科目完整视图（包含科目表和公司代码数据）
CREATE OR REPLACE VIEW vw_gl_accounts_full AS
SELECT
    ga.id,
    ga.chart_code,
    ga.account_code,
    ga.account_name,
    ga.account_name_long,
    ga.account_nature,
    ga.account_category,
    ga.account_level,
    ga.parent_account,
    ga.is_leaf_account,
    ga.full_path,
    ga.is_postable,
    ga.is_cost_element,
    ga.status,
    coa.chart_name,
    coa.accounting_standard,
    cca.company_code,
    cca.posting_blocked,
    cca.reconciliation_account_type,
    ga.created_at,
    ga.changed_at
FROM gl_accounts ga
INNER JOIN chart_of_accounts coa ON ga.chart_code = coa.chart_code
LEFT JOIN company_code_accounts cca ON ga.chart_code = cca.chart_code
    AND ga.account_code = cca.account_code;

-- 可过账科目视图
CREATE OR REPLACE VIEW vw_postable_accounts AS
SELECT
    chart_code,
    account_code,
    account_name,
    account_nature,
    account_level,
    parent_account,
    balance_indicator,
    status
FROM gl_accounts
WHERE is_postable = TRUE
  AND status = 'ACTIVE'
  AND (valid_to IS NULL OR valid_to >= CURRENT_DATE);

-- 科目层级树视图（使用递归 CTE）
CREATE OR REPLACE VIEW vw_account_hierarchy AS
WITH RECURSIVE account_tree AS (
    -- 根节点（一级科目）
    SELECT
        chart_code,
        account_code,
        account_name,
        parent_account,
        account_level,
        1 as depth,
        account_code as root_account,
        account_code::TEXT as path
    FROM gl_accounts
    WHERE parent_account IS NULL OR parent_account = ''

    UNION ALL

    -- 递归查找子节点
    SELECT
        ga.chart_code,
        ga.account_code,
        ga.account_name,
        ga.parent_account,
        ga.account_level,
        at.depth + 1,
        at.root_account,
        at.path || '/' || ga.account_code
    FROM gl_accounts ga
    INNER JOIN account_tree at ON ga.parent_account = at.account_code
        AND ga.chart_code = at.chart_code
    WHERE at.depth < 10  -- 防止无限递归
)
SELECT * FROM account_tree;

-- ============================================================================
-- 初始数据 (Initial Data)
-- ============================================================================

-- 插入默认科目表
INSERT INTO chart_of_accounts (chart_code, chart_name, description, accounting_standard, country_code, language, account_length, segment_structure)
VALUES
    ('CN01', '中国会计准则科目表', '中华人民共和国企业会计准则科目表', 'CN_GAAP', 'CN', 'ZH', 10, '4-2-2-2'),
    ('IFRS', 'IFRS 国际财务报告准则科目表', 'International Financial Reporting Standards COA', 'IFRS', 'XX', 'EN', 10, '4-2-2-2'),
    ('USGP', '美国公认会计准则科目表', 'US Generally Accepted Accounting Principles COA', 'US_GAAP', 'US', 'EN', 10, '4-2-2-2')
ON CONFLICT (chart_code) DO NOTHING;

-- 插入默认科目组（中国会计准则）
INSERT INTO account_groups (chart_code, group_code, group_name, account_nature, number_range_from, number_range_to)
VALUES
    ('CN01', 'ASSET', '资产类', 'ASSET', '1000000000', '1999999999'),
    ('CN01', 'LIAB', '负债类', 'LIABILITY', '2000000000', '2999999999'),
    ('CN01', 'EQUITY', '所有者权益类', 'EQUITY', '3000000000', '3999999999'),
    ('CN01', 'COST', '成本类', 'EXPENSE', '4000000000', '4999999999'),
    ('CN01', 'PLOSS', '损益类', 'PROFIT_LOSS', '5000000000', '6999999999')
ON CONFLICT (chart_code, group_code) DO NOTHING;

-- 插入示例科目（中国会计准则一级科目）
INSERT INTO gl_accounts (chart_code, account_code, account_name, account_nature, account_category, account_level, is_postable, balance_indicator, account_group)
VALUES
    ('CN01', '1001000000', '库存现金', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1002000000', '银行存款', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1012000000', '其他货币资金', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1101000000', '交易性金融资产', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1121000000', '应收票据', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1122000000', '应收账款', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1123000000', '预付账款', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1401000000', '材料采购', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1402000000', '在途物资', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '1403000000', '原材料', 'ASSET', 'BALANCE_SHEET', 1, TRUE, 'D', 'ASSET'),
    ('CN01', '2001000000', '短期借款', 'LIABILITY', 'BALANCE_SHEET', 1, TRUE, 'C', 'LIAB'),
    ('CN01', '2201000000', '应付票据', 'LIABILITY', 'BALANCE_SHEET', 1, TRUE, 'C', 'LIAB'),
    ('CN01', '2202000000', '应付账款', 'LIABILITY', 'BALANCE_SHEET', 1, TRUE, 'C', 'LIAB'),
    ('CN01', '4001000000', '实收资本', 'EQUITY', 'BALANCE_SHEET', 1, TRUE, 'C', 'EQUITY'),
    ('CN01', '4002000000', '资本公积', 'EQUITY', 'BALANCE_SHEET', 1, TRUE, 'C', 'EQUITY'),
    ('CN01', '6001000000', '主营业务收入', 'REVENUE', 'INCOME_STATEMENT', 1, TRUE, 'C', 'PLOSS'),
    ('CN01', '6051000000', '其他业务收入', 'REVENUE', 'INCOME_STATEMENT', 1, TRUE, 'C', 'PLOSS'),
    ('CN01', '6401000000', '主营业务成本', 'EXPENSE', 'INCOME_STATEMENT', 1, TRUE, 'D', 'PLOSS'),
    ('CN01', '6402000000', '其他业务成本', 'EXPENSE', 'INCOME_STATEMENT', 1, TRUE, 'D', 'PLOSS'),
    ('CN01', '6601000000', '销售费用', 'EXPENSE', 'INCOME_STATEMENT', 1, TRUE, 'D', 'PLOSS'),
    ('CN01', '6602000000', '管理费用', 'EXPENSE', 'INCOME_STATEMENT', 1, TRUE, 'D', 'PLOSS'),
    ('CN01', '6603000000', '财务费用', 'EXPENSE', 'INCOME_STATEMENT', 1, TRUE, 'D', 'PLOSS')
ON CONFLICT (chart_code, account_code) DO NOTHING;

-- 插入科目文本（中文+英文）
INSERT INTO account_texts (chart_code, account_code, language_code, short_text, medium_text, long_text)
VALUES
    ('CN01', '1001000000', 'ZH', '库存现金', '库存现金', '企业的库存现金'),
    ('CN01', '1001000000', 'EN', 'Cash on Hand', 'Cash on Hand', 'Cash on hand in the enterprise'),
    ('CN01', '1002000000', 'ZH', '银行存款', '银行存款', '企业存入银行或其他金融机构的各种款项'),
    ('CN01', '1002000000', 'EN', 'Bank Deposits', 'Bank Deposits', 'Various deposits in banks or other financial institutions'),
    ('CN01', '1122000000', 'ZH', '应收账款', '应收账款', '企业因销售商品、提供劳务等经营活动应收取的款项'),
    ('CN01', '1122000000', 'EN', 'Accounts Receivable', 'Accounts Receivable', 'Amounts due from customers for goods sold or services rendered'),
    ('CN01', '2202000000', 'ZH', '应付账款', '应付账款', '企业因购买材料、商品和接受劳务供应等经营活动应支付的款项'),
    ('CN01', '2202000000', 'EN', 'Accounts Payable', 'Accounts Payable', 'Amounts owed to suppliers for goods or services purchased'),
    ('CN01', '6001000000', 'ZH', '主营业务收入', '主营业务收入', '企业确认的销售商品、提供劳务等主营业务的收入'),
    ('CN01', '6001000000', 'EN', 'Revenue', 'Operating Revenue', 'Revenue from primary business operations'),
    ('CN01', '6401000000', 'ZH', '主营业务成本', '主营业务成本', '企业确认销售商品、提供劳务等主营业务收入时应结转的成本'),
    ('CN01', '6401000000', 'EN', 'COGS', 'Cost of Goods Sold', 'Cost of goods sold in primary business operations')
ON CONFLICT (chart_code, account_code, language_code) DO NOTHING;

-- ============================================================================
-- 函数 (Functions)
-- ============================================================================

-- 函数：验证科目是否可用
CREATE OR REPLACE FUNCTION fn_validate_account(
    p_chart_code VARCHAR(4),
    p_account_code VARCHAR(10),
    p_posting_date DATE DEFAULT CURRENT_DATE
) RETURNS TABLE (
    is_valid BOOLEAN,
    exists BOOLEAN,
    is_active BOOLEAN,
    is_postable BOOLEAN,
    error_message TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        CASE
            WHEN ga.id IS NULL THEN FALSE
            WHEN ga.status != 'ACTIVE' THEN FALSE
            WHEN ga.is_postable = FALSE THEN FALSE
            WHEN ga.valid_from IS NOT NULL AND ga.valid_from > p_posting_date THEN FALSE
            WHEN ga.valid_to IS NOT NULL AND ga.valid_to < p_posting_date THEN FALSE
            ELSE TRUE
        END as is_valid,
        ga.id IS NOT NULL as exists,
        ga.status = 'ACTIVE' as is_active,
        COALESCE(ga.is_postable, FALSE) as is_postable,
        CASE
            WHEN ga.id IS NULL THEN '科目不存在'
            WHEN ga.status != 'ACTIVE' THEN '科目未激活'
            WHEN ga.is_postable = FALSE THEN '科目不可过账'
            WHEN ga.valid_from IS NOT NULL AND ga.valid_from > p_posting_date THEN '科目尚未生效'
            WHEN ga.valid_to IS NOT NULL AND ga.valid_to < p_posting_date THEN '科目已过期'
            ELSE NULL
        END as error_message
    FROM gl_accounts ga
    WHERE ga.chart_code = p_chart_code
      AND ga.account_code = p_account_code;
END;
$$ LANGUAGE plpgsql;

-- 函数：获取科目完整路径
CREATE OR REPLACE FUNCTION fn_get_account_path(
    p_chart_code VARCHAR(4),
    p_account_code VARCHAR(10)
) RETURNS TEXT AS $$
DECLARE
    v_path TEXT := '';
    v_current_account VARCHAR(10) := p_account_code;
    v_parent_account VARCHAR(10);
    v_counter INT := 0;
BEGIN
    LOOP
        SELECT parent_account INTO v_parent_account
        FROM gl_accounts
        WHERE chart_code = p_chart_code
          AND account_code = v_current_account;

        IF v_path = '' THEN
            v_path := v_current_account;
        ELSE
            v_path := v_current_account || '/' || v_path;
        END IF;

        EXIT WHEN v_parent_account IS NULL OR v_parent_account = '';

        v_current_account := v_parent_account;
        v_counter := v_counter + 1;

        -- 防止无限循环
        IF v_counter > 10 THEN
            EXIT;
        END IF;
    END LOOP;

    RETURN v_path;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 触发器 (Triggers)
-- ============================================================================

-- 触发器函数：自动更新科目层级路径
CREATE OR REPLACE FUNCTION trg_update_account_path()
RETURNS TRIGGER AS $$
BEGIN
    -- 计算完整路径
    NEW.full_path := fn_get_account_path(NEW.chart_code, NEW.account_code);

    -- 计算深度
    NEW.depth := array_length(string_to_array(NEW.full_path, '/'), 1) - 1;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_gl_accounts_path
    BEFORE INSERT OR UPDATE ON gl_accounts
    FOR EACH ROW
    EXECUTE FUNCTION trg_update_account_path();

-- 触发器函数：更新修改时间
CREATE OR REPLACE FUNCTION trg_update_changed_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.changed_at := NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_gl_accounts_changed_at
    BEFORE UPDATE ON gl_accounts
    FOR EACH ROW
    EXECUTE FUNCTION trg_update_changed_at();

CREATE TRIGGER trg_company_code_accounts_changed_at
    BEFORE UPDATE ON company_code_accounts
    FOR EACH ROW
    EXECUTE FUNCTION trg_update_changed_at();

CREATE TRIGGER trg_account_groups_changed_at
    BEFORE UPDATE ON account_groups
    FOR EACH ROW
    EXECUTE FUNCTION trg_update_changed_at();

-- ============================================================================
-- 注释 (Comments)
-- ============================================================================

COMMENT ON TABLE chart_of_accounts IS '科目表主表 - 定义不同会计准则的科目表';
COMMENT ON TABLE gl_accounts IS '科目主数据表 - 存储会计科目主数据（对应 SAP SKA1）';
COMMENT ON TABLE company_code_accounts IS '公司代码级科目数据表 - 存储科目在公司代码级别的控制数据（对应 SAP SKB1）';
COMMENT ON TABLE account_texts IS '科目文本表 - 支持多语言的科目描述（对应 SAP SKAT）';
COMMENT ON TABLE account_groups IS '科目组表 - 科目分组和编号范围控制';
COMMENT ON TABLE account_validation_rules IS '科目验证规则表 - 定义科目的验证规则';

COMMENT ON COLUMN gl_accounts.account_code IS '科目代码 - 10位数字，支持分段编码（如 4-2-2-2）';
COMMENT ON COLUMN gl_accounts.full_path IS '完整路径 - 从根节点到当前科目的完整路径（如 1000/1001/100101）';
COMMENT ON COLUMN gl_accounts.is_postable IS '是否可过账 - TRUE表示末级科目可直接过账，FALSE表示仅用于汇总';
COMMENT ON COLUMN gl_accounts.balance_indicator IS '余额方向 - D=借方余额（资产/费用），C=贷方余额（负债/权益/收入）';
