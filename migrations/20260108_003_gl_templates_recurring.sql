-- ============================================================================
-- GL Service: Templates and Recurring Entries
-- 描述: 凭证模板和重复凭证表
-- ============================================================================

-- 重复频率枚举
CREATE TYPE recurring_frequency AS ENUM (
    'DAILY',
    'WEEKLY',
    'MONTHLY',
    'QUARTERLY',
    'SEMI_ANNUALLY',
    'ANNUALLY',
    'CUSTOM'
);

-- ============================================================================
-- 凭证模板表
-- ============================================================================
CREATE TABLE journal_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 模板信息
    template_name VARCHAR(100) NOT NULL,
    template_description VARCHAR(500),
    company_code VARCHAR(4) NOT NULL,
    
    -- 模板内容 (JSON 格式存储凭证结构)
    template_data JSONB NOT NULL,
    
    -- 是否公共模板
    is_public BOOLEAN DEFAULT FALSE,
    
    -- 审计字段
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uk_template_name 
        UNIQUE (company_code, template_name)
);

CREATE INDEX idx_template_company ON journal_templates(company_code);
CREATE INDEX idx_template_creator ON journal_templates(created_by);

-- ============================================================================
-- 重复凭证表
-- ============================================================================
CREATE TABLE recurring_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 基本信息
    entry_name VARCHAR(100) NOT NULL,
    company_code VARCHAR(4) NOT NULL,
    
    -- 重复凭证模板 (JSON 格式)
    entry_data JSONB NOT NULL,
    
    -- 频率设置
    frequency recurring_frequency NOT NULL,
    custom_interval_days INT, -- 自定义间隔天数
    
    -- 执行日期
    first_run_date DATE NOT NULL,
    last_run_date DATE,
    next_run_date DATE NOT NULL,
    end_date DATE, -- 可选结束日期
    
    -- 执行统计
    execution_count INT DEFAULT 0,
    max_executions INT, -- 可选最大执行次数
    
    -- 状态
    is_active BOOLEAN DEFAULT TRUE,
    
    -- 审计字段
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recurring_company ON recurring_entries(company_code);
CREATE INDEX idx_recurring_next_run ON recurring_entries(next_run_date) WHERE is_active = TRUE;

-- ============================================================================
-- 凭证附件表
-- ============================================================================
CREATE TABLE journal_attachments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- 外键
    header_id UUID NOT NULL REFERENCES journal_entry_headers(id) ON DELETE CASCADE,
    
    -- 附件信息
    file_name VARCHAR(255) NOT NULL,
    file_type VARCHAR(50),
    file_size BIGINT,
    storage_path VARCHAR(500) NOT NULL,
    
    -- 描述
    description VARCHAR(500),
    
    -- 审计字段
    uploaded_by UUID NOT NULL,
    uploaded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachment_header ON journal_attachments(header_id);
