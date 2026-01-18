-- Migration: Add Transaction Type Fields (VRGNG)
-- Description: 添加业务交易类型字段，用于区分不同业务场景
-- Date: 2026-01-18

-- ====================================================================================
-- 1. 添加业务交易类型字段到凭证行表
-- ====================================================================================

ALTER TABLE journal_entry_lines
ADD COLUMN IF NOT EXISTS transaction_type VARCHAR(4),
ADD COLUMN IF NOT EXISTS reference_transaction_type VARCHAR(5),
ADD COLUMN IF NOT EXISTS trading_partner_company VARCHAR(4);

-- 添加注释
COMMENT ON COLUMN journal_entry_lines.transaction_type IS 'VRGNG 业务交易类型（如：RV-销售发票、WE-采购收货）';
COMMENT ON COLUMN journal_entry_lines.reference_transaction_type IS 'AWTYP 参考交易类型（源系统类型：VBRK-销售、MKPF-物料凭证）';
COMMENT ON COLUMN journal_entry_lines.trading_partner_company IS 'VBUND 交易伙伴公司代码（集团内部交易）';

-- ====================================================================================
-- 2. 创建业务交易类型索引（性能优化）
-- ====================================================================================

-- 业务交易类型查询索引
CREATE INDEX IF NOT EXISTS idx_journal_lines_transaction_type
ON journal_entry_lines(transaction_type);

-- 参考交易类型查询索引
CREATE INDEX IF NOT EXISTS idx_journal_lines_ref_transaction_type
ON journal_entry_lines(reference_transaction_type);

-- 交易伙伴公司查询索引
CREATE INDEX IF NOT EXISTS idx_journal_lines_trading_partner
ON journal_entry_lines(trading_partner_company);

-- 复合索引（业务分析）
CREATE INDEX IF NOT EXISTS idx_journal_lines_business_analysis
ON journal_entry_lines(company_code, fiscal_year, transaction_type, reference_transaction_type);

-- ====================================================================================
-- 3. 创建业务交易类型参考表（可选）
-- ====================================================================================

-- 业务交易类型主数据表
CREATE TABLE IF NOT EXISTS transaction_type_master (
  transaction_type VARCHAR(4) PRIMARY KEY,
  description VARCHAR(100) NOT NULL,
  category VARCHAR(20) NOT NULL,  -- SALES, PURCHASE, ASSET, FINANCE, OTHER
  source_system VARCHAR(10),      -- SD, MM, AA, FI, etc.
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE transaction_type_master IS '业务交易类型主数据表';

-- 插入常见的业务交易类型
INSERT INTO transaction_type_master (transaction_type, description, category, source_system) VALUES
-- 销售业务 (SD - Sales & Distribution)
('RV', '销售发票 (Sales Invoice)', 'SALES', 'SD'),
('RD', '销售贷项凭证 (Sales Credit Memo)', 'SALES', 'SD'),
('DR', '销售借项凭证 (Sales Debit Memo)', 'SALES', 'SD'),
('DG', '销售退货 (Sales Return)', 'SALES', 'SD'),
('DZ', '销售折扣 (Sales Discount)', 'SALES', 'SD'),

-- 采购业务 (MM - Materials Management)
('WE', '采购收货 (Goods Receipt)', 'PURCHASE', 'MM'),
('RE', '采购发票 (Purchase Invoice)', 'PURCHASE', 'MM'),
('WA', '采购退货 (Goods Return)', 'PURCHASE', 'MM'),
('KR', '供应商贷项凭证 (Vendor Credit Memo)', 'PURCHASE', 'MM'),
('KG', '供应商借项凭证 (Vendor Debit Memo)', 'PURCHASE', 'MM'),

-- 资产业务 (AA - Asset Accounting)
('AA', '资产购置 (Asset Acquisition)', 'ASSET', 'AA'),
('AB', '资产折旧 (Asset Depreciation)', 'ASSET', 'AA'),
('AV', '资产处置 (Asset Retirement)', 'ASSET', 'AA'),
('AT', '资产转移 (Asset Transfer)', 'ASSET', 'AA'),

-- 财务业务 (FI - Financial Accounting)
('SA', '总账凭证 (G/L Account Posting)', 'FINANCE', 'FI'),
('ZP', '付款凭证 (Payment)', 'FINANCE', 'FI'),
('DZ', '收款凭证 (Receipt)', 'FINANCE', 'FI'),
('KZ', '银行对账 (Bank Reconciliation)', 'FINANCE', 'FI'),
('KU', '汇兑损益 (Foreign Exchange)', 'FINANCE', 'FI'),

-- 其他业务
('UM', '重分类 (Reclassification)', 'OTHER', 'FI'),
('AB', '期末结转 (Period-End Closing)', 'OTHER', 'FI')
ON CONFLICT (transaction_type) DO NOTHING;

-- 参考交易类型主数据表
CREATE TABLE IF NOT EXISTS reference_transaction_type_master (
  reference_transaction_type VARCHAR(5) PRIMARY KEY,
  description VARCHAR(100) NOT NULL,
  source_table VARCHAR(30),       -- SAP 源表名
  source_system VARCHAR(10),      -- SD, MM, AA, FI, etc.
  is_active BOOLEAN DEFAULT true,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

COMMENT ON TABLE reference_transaction_type_master IS '参考交易类型主数据表';

-- 插入常见的参考交易类型
INSERT INTO reference_transaction_type_master (reference_transaction_type, description, source_table, source_system) VALUES
-- 销售相关
('VBRK', '销售凭证抬头 (Billing Document Header)', 'VBRK', 'SD'),
('VBRP', '销售凭证行项目 (Billing Document Item)', 'VBRP', 'SD'),
('VBAK', '销售订单抬头 (Sales Order Header)', 'VBAK', 'SD'),
('VBAP', '销售订单行项目 (Sales Order Item)', 'VBAP', 'SD'),

-- 采购相关
('MKPF', '物料凭证抬头 (Material Document Header)', 'MKPF', 'MM'),
('MSEG', '物料凭证行项目 (Material Document Segment)', 'MSEG', 'MM'),
('EKKO', '采购订单抬头 (Purchase Order Header)', 'EKKO', 'MM'),
('EKPO', '采购订单行项目 (Purchase Order Item)', 'EKPO', 'MM'),
('RBKP', '发票凭证抬头 (Invoice Document Header)', 'RBKP', 'MM'),

-- 资产相关
('ANLA', '资产主数据 (Asset Master Record)', 'ANLA', 'AA'),
('ANLC', '资产价值字段 (Asset Value Fields)', 'ANLC', 'AA'),

-- 财务相关
('BKPF', '会计凭证抬头 (Accounting Document Header)', 'BKPF', 'FI'),
('BSEG', '会计凭证行项目 (Accounting Document Segment)', 'BSEG', 'FI'),
('REGUH', '付款凭证 (Payment Document)', 'REGUH', 'FI')
ON CONFLICT (reference_transaction_type) DO NOTHING;

-- ====================================================================================
-- 4. 创建业务交易类型统计视图
-- ====================================================================================

-- 业务交易类型汇总视图
CREATE OR REPLACE VIEW v_transaction_type_summary AS
SELECT
  jel.company_code,
  jel.fiscal_year,
  jel.transaction_type,
  ttm.description as transaction_type_description,
  ttm.category as business_category,
  jel.reference_transaction_type,
  rttm.description as reference_type_description,
  COUNT(*) as transaction_count,
  SUM(jel.amount_in_local_currency) as total_amount,
  COUNT(DISTINCT jel.document_number) as document_count,
  MIN(je.document_date) as earliest_date,
  MAX(je.document_date) as latest_date
FROM journal_entry_lines jel
JOIN journal_entries je ON jel.journal_entry_id = je.id
LEFT JOIN transaction_type_master ttm ON jel.transaction_type = ttm.transaction_type
LEFT JOIN reference_transaction_type_master rttm ON jel.reference_transaction_type = rttm.reference_transaction_type
WHERE jel.transaction_type IS NOT NULL
GROUP BY
  jel.company_code,
  jel.fiscal_year,
  jel.transaction_type,
  ttm.description,
  ttm.category,
  jel.reference_transaction_type,
  rttm.description;

COMMENT ON VIEW v_transaction_type_summary IS '业务交易类型汇总视图';

-- 业务类别汇总视图
CREATE OR REPLACE VIEW v_business_category_summary AS
SELECT
  jel.company_code,
  jel.fiscal_year,
  ttm.category as business_category,
  COUNT(*) as transaction_count,
  SUM(jel.amount_in_local_currency) as total_amount,
  COUNT(DISTINCT jel.transaction_type) as unique_transaction_types,
  COUNT(DISTINCT jel.document_number) as document_count
FROM journal_entry_lines jel
LEFT JOIN transaction_type_master ttm ON jel.transaction_type = ttm.transaction_type
WHERE jel.transaction_type IS NOT NULL
GROUP BY
  jel.company_code,
  jel.fiscal_year,
  ttm.category;

COMMENT ON VIEW v_business_category_summary IS '业务类别汇总视图';

-- 集团内部交易视图
CREATE OR REPLACE VIEW v_intercompany_transactions AS
SELECT
  jel.company_code,
  jel.trading_partner_company,
  jel.fiscal_year,
  jel.transaction_type,
  ttm.description as transaction_type_description,
  COUNT(*) as transaction_count,
  SUM(jel.amount_in_local_currency) as total_amount,
  COUNT(DISTINCT jel.document_number) as document_count
FROM journal_entry_lines jel
LEFT JOIN transaction_type_master ttm ON jel.transaction_type = ttm.transaction_type
WHERE jel.trading_partner_company IS NOT NULL
  AND jel.trading_partner_company != jel.company_code
GROUP BY
  jel.company_code,
  jel.trading_partner_company,
  jel.fiscal_year,
  jel.transaction_type,
  ttm.description;

COMMENT ON VIEW v_intercompany_transactions IS '集团内部交易视图';

-- ====================================================================================
-- 5. 创建业务交易类型验证函数
-- ====================================================================================

-- 验证业务交易类型的有效性
CREATE OR REPLACE FUNCTION validate_transaction_type(
  p_transaction_type VARCHAR(4)
) RETURNS BOOLEAN AS $$
BEGIN
  -- 如果为空，认为有效（可选字段）
  IF p_transaction_type IS NULL OR p_transaction_type = '' THEN
    RETURN TRUE;
  END IF;

  -- 检查是否存在于主数据表中
  RETURN EXISTS (
    SELECT 1 FROM transaction_type_master
    WHERE transaction_type = p_transaction_type
      AND is_active = true
  );
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION validate_transaction_type IS '验证业务交易类型的有效性';

-- 验证参考交易类型的有效性
CREATE OR REPLACE FUNCTION validate_reference_transaction_type(
  p_reference_transaction_type VARCHAR(5)
) RETURNS BOOLEAN AS $$
BEGIN
  -- 如果为空，认为有效（可选字段）
  IF p_reference_transaction_type IS NULL OR p_reference_transaction_type = '' THEN
    RETURN TRUE;
  END IF;

  -- 检查是否存在于主数据表中
  RETURN EXISTS (
    SELECT 1 FROM reference_transaction_type_master
    WHERE reference_transaction_type = p_reference_transaction_type
      AND is_active = true
  );
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION validate_reference_transaction_type IS '验证参考交易类型的有效性';

-- ====================================================================================
-- 6. 创建业务交易类型统计表（可选 - 用于性能优化）
-- ====================================================================================

-- 业务交易类型每日统计表
CREATE TABLE IF NOT EXISTS transaction_type_daily_stats (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  company_code VARCHAR(4) NOT NULL,
  fiscal_year INT NOT NULL,
  transaction_date DATE NOT NULL,
  transaction_type VARCHAR(4) NOT NULL,
  business_category VARCHAR(20),
  transaction_count INT DEFAULT 0,
  total_amount DECIMAL(15,2) DEFAULT 0,
  document_count INT DEFAULT 0,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(company_code, fiscal_year, transaction_date, transaction_type)
);

COMMENT ON TABLE transaction_type_daily_stats IS '业务交易类型每日统计表';

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_transaction_daily_stats_date
ON transaction_type_daily_stats(company_code, fiscal_year, transaction_date);

CREATE INDEX IF NOT EXISTS idx_transaction_daily_stats_type
ON transaction_type_daily_stats(transaction_type, transaction_date);

-- ====================================================================================
-- 7. 数据迁移（可选）
-- ====================================================================================

-- 如果需要为现有数据设置默认值
-- UPDATE journal_entry_lines
-- SET transaction_type = 'SA'  -- 默认为总账凭证
-- WHERE transaction_type IS NULL;

-- ====================================================================================
-- 8. 创建触发器（可选 - 自动更新统计表）
-- ====================================================================================

-- 自动更新每日统计表的触发器函数
CREATE OR REPLACE FUNCTION update_transaction_type_daily_stats()
RETURNS TRIGGER AS $$
BEGIN
  -- 插入或更新统计数据
  INSERT INTO transaction_type_daily_stats (
    company_code,
    fiscal_year,
    transaction_date,
    transaction_type,
    business_category,
    transaction_count,
    total_amount,
    document_count
  )
  SELECT
    NEW.company_code,
    NEW.fiscal_year,
    (SELECT document_date FROM journal_entries WHERE id = NEW.journal_entry_id),
    NEW.transaction_type,
    (SELECT category FROM transaction_type_master WHERE transaction_type = NEW.transaction_type),
    1,
    NEW.amount_in_local_currency,
    1
  WHERE NEW.transaction_type IS NOT NULL
  ON CONFLICT (company_code, fiscal_year, transaction_date, transaction_type)
  DO UPDATE SET
    transaction_count = transaction_type_daily_stats.transaction_count + 1,
    total_amount = transaction_type_daily_stats.total_amount + EXCLUDED.total_amount,
    document_count = transaction_type_daily_stats.document_count + 1,
    updated_at = CURRENT_TIMESTAMP;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 创建触发器（可选 - 根据性能需求决定是否启用）
-- CREATE TRIGGER trg_update_transaction_type_stats
-- AFTER INSERT ON journal_entry_lines
-- FOR EACH ROW
-- EXECUTE FUNCTION update_transaction_type_daily_stats();

-- ====================================================================================
-- 完成
-- ====================================================================================

-- 验证 Migration
DO $$
BEGIN
  -- 检查字段是否添加成功
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.columns
    WHERE table_name = 'journal_entry_lines'
      AND column_name = 'transaction_type'
  ) THEN
    RAISE EXCEPTION 'Migration failed: transaction_type column not added';
  END IF;

  -- 检查主数据表是否创建成功
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_name = 'transaction_type_master'
  ) THEN
    RAISE EXCEPTION 'Migration failed: transaction_type_master table not created';
  END IF;

  RAISE NOTICE 'Migration 20260118000004_add_transaction_type completed successfully';
END $$;
