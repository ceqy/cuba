-- Universal Journal Table in ClickHouse
-- 使用 ReplacingMergeTree 引擎，支持基于version的去重（模拟Upsert）
-- 排序键 (ORDER BY) 决定了数据在磁盘上的物理排序，也是主键
CREATE TABLE IF NOT EXISTS universal_journal_entries (
    -- 联合主键部分
    ledger String,
    company_code String,
    fiscal_year UInt32,
    document_number String,
    document_line UInt32,
    
    -- 核心维度
    document_type String,
    posting_date Date,
    fiscal_period UInt32,
    gl_account String,
    cost_center String,
    profit_center String,
    segment String,
    business_area String,
    controlling_area String,
    
    -- 度量值 (Decimal128 精度高，适合财务金额)
    amount_in_local_currency Decimal128(2),
    amount_in_document_currency Decimal128(2),
    amount_in_group_currency Nullable(Decimal128(2)),
    
    -- 货币及其他
    local_currency String,
    document_currency String,
    group_currency Nullable(String),
    
    -- 扩展维度
    customer String,
    vendor String,
    material String,
    plant String,
    
    -- 源数据追踪
    source_module String,
    created_at DateTime,
    created_by String,
    
    -- 版本控制 (用于 ReplacingMergeTree 的去重逻辑)
    version UInt64 DEFAULT toUnixTimestamp(now())
) ENGINE = ReplacingMergeTree(version)
ORDER BY (ledger, company_code, fiscal_year, posting_date, document_number, document_line)
PARTITION BY toYYYYMM(posting_date);
