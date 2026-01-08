-- ============================================================================
-- GL Service: Journal Entry Core Tables
-- 描述: 总账凭证核心表结构，参考 SAP ACDOCA/BKPF/BSEG 设计
-- ============================================================================

-- 凭证状态枚举
CREATE TYPE journal_entry_status AS ENUM (
    'DRAFT',           -- 草稿
    'PARKED',          -- 暂存
    'PENDING_APPROVAL', -- 待审批
    'APPROVED',        -- 已审批
    'POSTED',          -- 已过账
    'REVERSED',        -- 已冲销
    'CANCELLED'        -- 已取消
);

-- 凭证来源枚举
CREATE TYPE document_origin AS ENUM (
    'MANUAL',          -- 手工输入
    'BATCH_INPUT',     -- 批量输入
    'INTERFACE',       -- 接口导入
    'AUTOMATIC',       -- 自动生成
    'RECURRING',       -- 重复凭证
    'TEMPLATE',        -- 模板生成
    'PARKED'           -- 从停放凭证过账
);

-- ============================================================================
-- 凭证抬头表 (参考: BKPF)
-- ============================================================================
CREATE TABLE journal_entry_headers (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 业务键 (唯一标识)
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    document_number VARCHAR(20) NOT NULL,
    
    -- 凭证类型和日期
    document_type VARCHAR(4) NOT NULL,
    document_date DATE NOT NULL,
    posting_date DATE NOT NULL,
    fiscal_period INT NOT NULL,
    
    -- 货币信息
    currency VARCHAR(3) NOT NULL,
    exchange_rate DECIMAL(15, 6) DEFAULT 1.0,
    local_currency VARCHAR(3) DEFAULT 'CNY',
    
    -- 文本和参考
    header_text VARCHAR(100),
    reference_document VARCHAR(50),
    reference_key VARCHAR(50),
    
    -- 状态
    status journal_entry_status NOT NULL DEFAULT 'DRAFT',
    document_origin document_origin NOT NULL DEFAULT 'MANUAL',
    
    -- 冲销信息
    is_reversal BOOLEAN DEFAULT FALSE,
    reversal_document_id UUID REFERENCES journal_entry_headers(id),
    reversal_reason VARCHAR(100),
    
    -- 审批信息
    approval_status VARCHAR(20),
    approved_by UUID,
    approved_at TIMESTAMPTZ,
    
    -- 分类账
    ledger VARCHAR(4) DEFAULT '0L',
    
    -- 乐观锁版本
    version BIGINT NOT NULL DEFAULT 1,
    
    -- 审计字段
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- 业务键唯一约束
    CONSTRAINT uk_journal_entry_business_key 
        UNIQUE (company_code, fiscal_year, document_number)
);

-- 索引
CREATE INDEX idx_journal_entry_posting_date ON journal_entry_headers(posting_date);
CREATE INDEX idx_journal_entry_status ON journal_entry_headers(status);
CREATE INDEX idx_journal_entry_created_by ON journal_entry_headers(created_by);
CREATE INDEX idx_journal_entry_company_period ON journal_entry_headers(company_code, fiscal_year, fiscal_period);

-- ============================================================================
-- 凭证行项目表 (参考: BSEG)
-- ============================================================================
CREATE TABLE journal_entry_lines (
    -- 主键
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 外键
    header_id UUID NOT NULL REFERENCES journal_entry_headers(id) ON DELETE CASCADE,
    
    -- 行项目号
    line_number INT NOT NULL,
    
    -- 科目信息
    gl_account VARCHAR(10) NOT NULL,
    account_type VARCHAR(1) NOT NULL DEFAULT 'S', -- S=GL, D=Customer, K=Vendor
    customer_number VARCHAR(10),
    vendor_number VARCHAR(10),
    
    -- 金额 (凭证货币)
    amount_doc_currency DECIMAL(15, 2) NOT NULL,
    debit_credit_indicator VARCHAR(1) NOT NULL, -- S=Debit, H=Credit
    
    -- 金额 (本位币)
    amount_local_currency DECIMAL(15, 2),
    
    -- 成本对象
    cost_center VARCHAR(10),
    profit_center VARCHAR(10),
    internal_order VARCHAR(12),
    wbs_element VARCHAR(24),
    
    -- 业务区域
    business_area VARCHAR(4),
    functional_area VARCHAR(16),
    segment VARCHAR(10),
    
    -- 文本
    line_text VARCHAR(100),
    assignment VARCHAR(50),
    
    -- 清账信息
    clearing_status VARCHAR(20) DEFAULT 'OPEN',
    clearing_document_id UUID,
    clearing_date DATE,
    
    -- 支付信息
    payment_terms VARCHAR(4),
    baseline_date DATE,
    due_date DATE,
    
    -- 税务
    tax_code VARCHAR(4),
    tax_amount DECIMAL(15, 2),
    
    -- 行项目唯一约束
    CONSTRAINT uk_journal_entry_line 
        UNIQUE (header_id, line_number)
);

-- 索引
CREATE INDEX idx_journal_line_header ON journal_entry_lines(header_id);
CREATE INDEX idx_journal_line_gl_account ON journal_entry_lines(gl_account);
CREATE INDEX idx_journal_line_clearing ON journal_entry_lines(clearing_status);
CREATE INDEX idx_journal_line_cost_center ON journal_entry_lines(cost_center);

-- ============================================================================
-- 触发器: 自动更新 updated_at
-- ============================================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER tr_journal_entry_headers_updated
    BEFORE UPDATE ON journal_entry_headers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
