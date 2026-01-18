-- ============================================================================
-- Universal Journal (ACDOCA) 表创建脚本
-- 描述: 创建 Universal Journal 统一视图表，映射 SAP ACDOCA 表结构
-- ============================================================================

-- 创建 universal_journal_entries 表
CREATE TABLE IF NOT EXISTS universal_journal_entries (
    -- ============================================================================
    -- 主键字段 (Primary Key Fields)
    -- ============================================================================
    ledger VARCHAR(2) NOT NULL,                    -- RLDNR 分类账
    company_code VARCHAR(4) NOT NULL,              -- RBUKRS 公司代码
    fiscal_year INTEGER NOT NULL,                  -- GJAHR 会计年度
    document_number VARCHAR(10) NOT NULL,          -- BELNR 凭证号
    document_line INTEGER NOT NULL,                -- DOCLN 凭证行号

    -- ============================================================================
    -- 凭证抬头字段 (Document Header Fields)
    -- ============================================================================
    document_type VARCHAR(2) NOT NULL,             -- BLART 凭证类型
    document_date DATE NOT NULL,                   -- BLDAT 凭证日期
    posting_date DATE NOT NULL,                    -- BUDAT 过账日期
    fiscal_period INTEGER NOT NULL,                -- MONAT 会计期间
    reference_document VARCHAR(16),                -- XBLNR 参考凭证号
    header_text TEXT,                              -- BKTXT 凭证抬头文本
    document_currency VARCHAR(3) NOT NULL,         -- WAERS 凭证货币
    exchange_rate DECIMAL(19, 9),                  -- KURSF 汇率
    logical_system VARCHAR(10),                    -- AWSYS 逻辑系统
    transaction_code VARCHAR(20),                  -- TCODE 事务代码

    -- ============================================================================
    -- 行项目字段 (Line Item Fields)
    -- ============================================================================
    posting_key VARCHAR(2) NOT NULL,               -- BSCHL 过账码
    debit_credit_indicator VARCHAR(1) NOT NULL,    -- SHKZG 借贷标识（S-借方，H-贷方）
    account_type VARCHAR(1) NOT NULL,              -- KOART 账户类型
    gl_account VARCHAR(10) NOT NULL,               -- RACCT 总账科目
    business_partner VARCHAR(10),                  -- KUNNR/LIFNR 业务伙伴
    material VARCHAR(40),                          -- MATNR 物料号
    plant VARCHAR(4),                              -- WERKS 工厂
    item_text TEXT,                                -- SGTXT 行项目文本
    assignment_number VARCHAR(18),                 -- ZUONR 指派编号

    -- ============================================================================
    -- 金额字段 (Amount Fields)
    -- ============================================================================
    amount_in_document_currency DECIMAL(23, 2) NOT NULL, -- WRBTR 凭证货币金额
    amount_in_local_currency DECIMAL(23, 2) NOT NULL,    -- DMBTR 本位币金额
    amount_in_group_currency DECIMAL(23, 2),             -- DMBE2 集团货币金额
    amount_in_global_currency DECIMAL(23, 2),            -- DMBE3 全球货币金额
    amount_in_ledger_currency DECIMAL(23, 2),            -- HSL 分类账货币金额

    -- ============================================================================
    -- 数量字段 (Quantity Fields)
    -- ============================================================================
    quantity DECIMAL(23, 3),                       -- MENGE 数量
    quantity_unit VARCHAR(3),                      -- MEINS 单位

    -- ============================================================================
    -- 成本对象字段 (Cost Object Fields)
    -- ============================================================================
    cost_center VARCHAR(10),                       -- KOSTL 成本中心
    profit_center VARCHAR(10),                     -- PRCTR 利润中心
    segment VARCHAR(10),                           -- SEGMENT 段
    functional_area VARCHAR(16),                   -- FKBER 功能范围
    business_area VARCHAR(4),                      -- GSBER 业务范围
    controlling_area VARCHAR(4),                   -- KOKRS 控制范围
    internal_order VARCHAR(12),                    -- AUFNR 内部订单
    wbs_element VARCHAR(24),                       -- PS_PSP_PNR WBS 元素
    sales_order VARCHAR(10),                       -- VBELN 销售订单
    sales_order_item INTEGER,                      -- POSNR 销售订单行项目

    -- ============================================================================
    -- 税务字段 (Tax Fields)
    -- ============================================================================
    tax_code VARCHAR(2),                           -- MWSKZ 税码
    tax_jurisdiction VARCHAR(15),                  -- TXJCD 税收辖区
    tax_amount DECIMAL(23, 2),                     -- MWSTS 税额

    -- ============================================================================
    -- 清账字段 (Clearing Fields)
    -- ============================================================================
    clearing_document VARCHAR(10),                 -- AUGBL 清账凭证号
    clearing_date DATE,                            -- AUGDT 清账日期

    -- ============================================================================
    -- 付款字段 (Payment Fields)
    -- ============================================================================
    baseline_date DATE,                            -- ZFBDT 基准日期
    due_date DATE,                                 -- NETDT 到期日
    payment_terms VARCHAR(4),                      -- ZTERM 付款条件
    payment_method VARCHAR(1),                     -- ZLSCH 付款方式
    payment_block VARCHAR(1),                      -- ZLSPR 付款冻结
    house_bank VARCHAR(5),                         -- HBKID 内部银行账户

    -- ============================================================================
    -- 特殊总账字段 (Special G/L Fields)
    -- ============================================================================
    special_gl_indicator VARCHAR(1),               -- UMSKZ 特殊总账标识

    -- ============================================================================
    -- 发票参考字段 (Invoice Reference Fields)
    -- ============================================================================
    reference_document_number VARCHAR(10),         -- REBZG 参考凭证号
    reference_fiscal_year INTEGER,                 -- REBZJ 参考会计年度
    reference_line_item INTEGER,                   -- REBZZ 参考行项目号
    reference_document_type VARCHAR(2),            -- REBZT 参考凭证类型

    -- ============================================================================
    -- 业务交易类型字段 (Transaction Type Fields)
    -- ============================================================================
    transaction_type VARCHAR(4),                   -- VRGNG 业务交易类型
    reference_transaction_type VARCHAR(5),         -- AWTYP 参考交易类型
    reference_key_1 VARCHAR(50),                   -- AWREF 参考键 1
    reference_key_2 VARCHAR(10),                   -- AWORG 参考键 2
    reference_key_3 VARCHAR(10),                   -- AWSYS 参考键 3

    -- ============================================================================
    -- 组织维度字段 (Organizational Dimensions)
    -- ============================================================================
    financial_area VARCHAR(4),                     -- RFAREA 财务范围
    consolidation_unit VARCHAR(4),                 -- RUNIT 合并单位
    partner_company VARCHAR(6),                    -- VBUND 伙伴公司代码
    trading_partner VARCHAR(6),                    -- VKORG 交易伙伴

    -- ============================================================================
    -- 多币种字段 (Multi-Currency Fields)
    -- ============================================================================
    local_currency VARCHAR(3) NOT NULL,            -- RHCUR 本位币
    group_currency VARCHAR(3),                     -- RKCUR 集团货币
    global_currency VARCHAR(3),                    -- RTCUR 全球货币
    amount_in_object_currency DECIMAL(23, 2),      -- OSL 对象货币金额
    amount_in_profit_center_currency DECIMAL(23, 2), -- VSL 利润中心货币金额

    -- ============================================================================
    -- 催款字段 (Dunning Fields)
    -- ============================================================================
    dunning_key VARCHAR(1),                        -- MSCHL 催款码
    dunning_block VARCHAR(1),                      -- MANST 催款冻结
    last_dunning_date DATE,                        -- MADAT 上次催款日期
    dunning_level INTEGER,                         -- 催款级别

    -- ============================================================================
    -- 付款条件详细字段 (Payment Terms Detail)
    -- ============================================================================
    discount_days_1 INTEGER,                       -- ZBD1T 第一个折扣天数
    discount_days_2 INTEGER,                       -- ZBD2T 第二个折扣天数
    net_payment_days INTEGER,                      -- ZBD3T 净付款天数
    discount_percent_1 DECIMAL(5, 3),              -- ZBD1P 第一个折扣百分比
    discount_percent_2 DECIMAL(5, 3),              -- ZBD2P 第二个折扣百分比
    discount_amount DECIMAL(23, 2),                -- SKFBT 现金折扣金额

    -- ============================================================================
    -- 内部交易字段 (Internal Trading Fields)
    -- ============================================================================
    sending_cost_center VARCHAR(10),               -- SCNTR 发送成本中心
    partner_profit_center VARCHAR(10),             -- PPRCTR 伙伴利润中心
    sending_financial_area VARCHAR(4),             -- SFAREA 发送财务范围

    -- ============================================================================
    -- 科目分配字段 (Account Assignment Fields)
    -- ============================================================================
    account_assignment VARCHAR(3),                 -- KTOSL 科目分配

    -- ============================================================================
    -- 本地 GAAP 字段 (Local GAAP Fields)
    -- ============================================================================
    local_account VARCHAR(10),                     -- LOKKT 本地科目
    data_source VARCHAR(2),                        -- HRKFT 数据来源

    -- ============================================================================
    -- 字段拆分字段 (Field Split Fields)
    -- ============================================================================
    split_method VARCHAR(2),                       -- XSPLITMOD 拆分方法
    manual_split BOOLEAN DEFAULT FALSE,            -- MANSP 手工拆分标识

    -- ============================================================================
    -- 审计字段 (Audit Fields)
    -- ============================================================================
    created_by VARCHAR(12) NOT NULL,               -- USNAM 创建人
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- CPUDT 创建日期时间
    changed_by VARCHAR(12),                        -- AENAM 修改人
    changed_at TIMESTAMP,                          -- AEDAT 修改日期时间

    -- ============================================================================
    -- 来源模块标识 (Source Module Identifier)
    -- ============================================================================
    source_module VARCHAR(10) NOT NULL,            -- 来源模块

    -- ============================================================================
    -- 扩展字段 (Extension Fields)
    -- ============================================================================
    extension_fields JSONB,                        -- 扩展字段

    -- 主键约束
    PRIMARY KEY (ledger, company_code, fiscal_year, document_number, document_line)
);

-- ============================================================================
-- 创建索引以优化查询性能
-- ============================================================================

-- 过账日期索引（最常用的查询条件）
CREATE INDEX idx_uj_posting_date ON universal_journal_entries(posting_date);

-- 凭证日期索引
CREATE INDEX idx_uj_document_date ON universal_journal_entries(document_date);

-- 公司代码 + 会计年度索引
CREATE INDEX idx_uj_company_fiscal_year ON universal_journal_entries(company_code, fiscal_year);

-- 总账科目索引
CREATE INDEX idx_uj_gl_account ON universal_journal_entries(gl_account);

-- 业务伙伴索引
CREATE INDEX idx_uj_business_partner ON universal_journal_entries(business_partner) WHERE business_partner IS NOT NULL;

-- 成本中心索引
CREATE INDEX idx_uj_cost_center ON universal_journal_entries(cost_center) WHERE cost_center IS NOT NULL;

-- 利润中心索引
CREATE INDEX idx_uj_profit_center ON universal_journal_entries(profit_center) WHERE profit_center IS NOT NULL;

-- 段索引
CREATE INDEX idx_uj_segment ON universal_journal_entries(segment) WHERE segment IS NOT NULL;

-- 业务范围索引
CREATE INDEX idx_uj_business_area ON universal_journal_entries(business_area) WHERE business_area IS NOT NULL;

-- 清账状态索引（用于未清项查询）
CREATE INDEX idx_uj_clearing_status ON universal_journal_entries(clearing_document) WHERE clearing_document IS NULL;

-- 来源模块索引
CREATE INDEX idx_uj_source_module ON universal_journal_entries(source_module);

-- 凭证类型索引
CREATE INDEX idx_uj_document_type ON universal_journal_entries(document_type);

-- 特殊总账标识索引
CREATE INDEX idx_uj_special_gl ON universal_journal_entries(special_gl_indicator) WHERE special_gl_indicator IS NOT NULL;

-- 复合索引：公司代码 + 会计年度 + 过账日期（用于期间查询）
CREATE INDEX idx_uj_company_year_posting ON universal_journal_entries(company_code, fiscal_year, posting_date);

-- 复合索引：总账科目 + 过账日期（用于科目余额查询）
CREATE INDEX idx_uj_account_posting ON universal_journal_entries(gl_account, posting_date);

-- 全文搜索索引（用于文本搜索）
CREATE INDEX idx_uj_text_search ON universal_journal_entries USING gin(
    to_tsvector('simple', COALESCE(header_text, '') || ' ' || COALESCE(item_text, ''))
);

-- ============================================================================
-- 添加注释
-- ============================================================================

COMMENT ON TABLE universal_journal_entries IS 'Universal Journal (ACDOCA) 统一视图表 - 存储所有财务模块的凭证数据';
COMMENT ON COLUMN universal_journal_entries.ledger IS 'RLDNR - 分类账（0L-主账，1L/2L-非主账）';
COMMENT ON COLUMN universal_journal_entries.company_code IS 'RBUKRS - 公司代码';
COMMENT ON COLUMN universal_journal_entries.fiscal_year IS 'GJAHR - 会计年度';
COMMENT ON COLUMN universal_journal_entries.document_number IS 'BELNR - 凭证号';
COMMENT ON COLUMN universal_journal_entries.document_line IS 'DOCLN - 凭证行号';
COMMENT ON COLUMN universal_journal_entries.source_module IS '来源模块（GL/AP/AR/AA/MM/SD/CO/TR）';
