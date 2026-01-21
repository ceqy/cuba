-- 添加付款执行和付款条件详细字段
-- payment_execution: 付款执行信息（付款方式、冻结、优先级等）
-- payment_terms_detail: 付款条件详细信息（现金折扣计算）

-- 添加 JSONB 字段到 journal_entry_lines 表
ALTER TABLE journal_entry_lines ADD COLUMN IF NOT EXISTS payment_execution JSONB;
ALTER TABLE journal_entry_lines ADD COLUMN IF NOT EXISTS payment_terms_detail JSONB;

-- 付款方式索引（用于自动付款程序查询）
CREATE INDEX IF NOT EXISTS idx_jel_payment_method
    ON journal_entry_lines ((payment_execution->>'payment_method'))
    WHERE payment_execution IS NOT NULL;

-- 付款冻结索引（用于查询被冻结的付款）
CREATE INDEX IF NOT EXISTS idx_jel_payment_block
    ON journal_entry_lines ((payment_execution->>'payment_block'))
    WHERE payment_execution IS NOT NULL AND payment_execution->>'payment_block' IS NOT NULL;

-- 付款基准日索引（用于现金折扣到期日计算）
CREATE INDEX IF NOT EXISTS idx_jel_baseline_date
    ON journal_entry_lines ((payment_terms_detail->>'baseline_date'))
    WHERE payment_terms_detail IS NOT NULL;

-- 添加字段注释
COMMENT ON COLUMN journal_entry_lines.payment_execution IS '付款执行详细信息（JSONB）：payment_method(ZLSCH), house_bank(HBKID), partner_bank_type(BVTYP), payment_block(ZLSPR), payment_baseline_date(ZFBDT), payment_reference, payment_priority';
COMMENT ON COLUMN journal_entry_lines.payment_terms_detail IS '付款条件详细信息（JSONB）：baseline_date(ZFBDT), discount_days_1(ZBD1T), discount_days_2(ZBD2T), net_payment_days(ZBD3T), discount_percent_1(ZBD1P), discount_percent_2(ZBD2P), discount_amount(SKFBT)';
