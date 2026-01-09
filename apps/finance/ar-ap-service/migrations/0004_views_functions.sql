-- ============================================================================
-- AR/AP Service: Database Views and Helper Functions
-- 描述: 常用查询视图和辅助函数
-- ============================================================================

-- 视图：合并的伙伴信息（客户+供应商）
CREATE OR REPLACE VIEW v_partner_overview AS
SELECT 
    bp.partner_id,
    bp.partner_type,
    bp.name_org1,
    bp.name_last,
    bp.name_first,
    bp.country,
    c.customer_id,
    c.company_code AS customer_company_code,
    c.credit_limit AS customer_credit_limit,
    s.supplier_id,
    s.company_code AS supplier_company_code
FROM business_partners bp
LEFT JOIN customers c ON bp.partner_id = c.partner_id
LEFT JOIN suppliers s ON bp.partner_id = s.partner_id;

-- 视图：未清项汇总（按伙伴）
CREATE OR REPLACE VIEW v_open_items_summary AS
SELECT 
    partner_id,
    company_code,
    account_type,
    currency,
    COUNT(*) AS item_count,
    SUM(open_amount) AS total_open_amount,
    MIN(posting_date) AS earliest_posting_date,
    MAX(due_date) AS latest_due_date,
    COUNT(CASE WHEN due_date < CURRENT_DATE AND clearing_date IS NULL THEN 1 END) AS overdue_count
FROM open_items
WHERE clearing_date IS NULL
GROUP BY partner_id, company_code, account_type, currency;

-- 视图：账龄分析（实时）
CREATE OR REPLACE VIEW v_aging_analysis AS
SELECT 
    partner_id,
    company_code,
    account_type,
    currency,
    SUM(CASE 
        WHEN due_date IS NULL OR due_date >= CURRENT_DATE THEN open_amount 
        ELSE 0 
    END) AS current_amount,
    SUM(CASE 
        WHEN due_date < CURRENT_DATE AND due_date >= CURRENT_DATE - INTERVAL '30 days' THEN open_amount 
        ELSE 0 
    END) AS days_1_30,
    SUM(CASE 
        WHEN due_date < CURRENT_DATE - INTERVAL '30 days' AND due_date >= CURRENT_DATE - INTERVAL '60 days' THEN open_amount 
        ELSE 0 
    END) AS days_31_60,
    SUM(CASE 
        WHEN due_date < CURRENT_DATE - INTERVAL '60 days' AND due_date >= CURRENT_DATE - INTERVAL '90 days' THEN open_amount 
        ELSE 0 
    END) AS days_61_90,
    SUM(CASE 
        WHEN due_date < CURRENT_DATE - INTERVAL '90 days' THEN open_amount 
        ELSE 0 
    END) AS over_90_days
FROM open_items
WHERE clearing_date IS NULL
GROUP BY partner_id, company_code, account_type, currency;

-- 视图：客户信用额度使用情况
CREATE OR REPLACE VIEW v_customer_credit_exposure AS
SELECT 
    c.customer_id,
    c.company_code,
    c.credit_limit,
    c.credit_currency,
    COALESCE(SUM(oi.open_amount), 0) AS current_exposure,
    c.credit_limit - COALESCE(SUM(oi.open_amount), 0) AS available_credit,
    CASE 
        WHEN c.credit_limit IS NULL THEN 'NO_LIMIT'
        WHEN COALESCE(SUM(oi.open_amount), 0) > c.credit_limit THEN 'EXCEEDED'
        WHEN COALESCE(SUM(oi.open_amount), 0) / c.credit_limit > 0.9 THEN 'WARNING'
        ELSE 'OK'
    END AS credit_status
FROM customers c
LEFT JOIN open_items oi ON c.partner_id = oi.partner_id 
    AND c.company_code = oi.company_code 
    AND oi.account_type = 'CUSTOMER'
    AND oi.clearing_date IS NULL
GROUP BY c.customer_id, c.company_code, c.credit_limit, c.credit_currency;

-- 函数：计算账户余额
CREATE OR REPLACE FUNCTION get_account_balance(
    p_partner_id VARCHAR(10),
    p_company_code VARCHAR(4),
    p_currency VARCHAR(3) DEFAULT 'CNY'
)
RETURNS TABLE (
    debit_total DECIMAL(15,2),
    credit_total DECIMAL(15,2),
    balance DECIMAL(15,2),
    item_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        COALESCE(SUM(CASE WHEN amount > 0 THEN open_amount ELSE 0 END), 0) AS debit_total,
        COALESCE(SUM(CASE WHEN amount < 0 THEN ABS(open_amount) ELSE 0 END), 0) AS credit_total,
        COALESCE(SUM(open_amount), 0) AS balance,
        COUNT(*) AS item_count
    FROM open_items
    WHERE partner_id = p_partner_id
      AND company_code = p_company_code
      AND currency = p_currency
      AND clearing_date IS NULL;
END;
$$ LANGUAGE plpgsql;

-- 函数：获取逾期天数
CREATE OR REPLACE FUNCTION get_days_overdue(p_due_date DATE)
RETURNS INTEGER AS $$
BEGIN
    IF p_due_date IS NULL THEN
        RETURN 0;
    END IF;
    
    RETURN GREATEST(0, CURRENT_DATE - p_due_date);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- 函数：计算付款折扣
CREATE OR REPLACE FUNCTION calculate_payment_discount(
    p_amount DECIMAL(15,2),
    p_payment_terms VARCHAR(4),
    p_invoice_date DATE,
    p_payment_date DATE DEFAULT CURRENT_DATE
)
RETURNS DECIMAL(15,2) AS $$
DECLARE
    v_days_diff INTEGER;
    v_discount_rate DECIMAL(5,2);
BEGIN
    v_days_diff := p_payment_date - p_invoice_date;
    
    -- 简化折扣逻辑，实际应从配置表读取
    v_discount_rate := CASE 
        WHEN p_payment_terms = 'Z001' AND v_days_diff <= 10 THEN 2.00  -- 2%
        WHEN p_payment_terms = 'Z001' AND v_days_diff <= 30 THEN 1.00  -- 1%
        WHEN p_payment_terms = 'Z002' AND v_days_diff <= 7 THEN 3.00   -- 3%
        ELSE 0.00
    END;
    
    RETURN ROUND(p_amount * v_discount_rate / 100, 2);
END;
$$ LANGUAGE plpgs

ql;

-- 添加额外的约束
ALTER TABLE customers ADD CONSTRAINT chk_credit_limit_positive 
    CHECK (credit_limit IS NULL OR credit_limit >= 0);

ALTER TABLE open_items ADD CONSTRAINT chk_open_amount_lte_amount 
    CHECK (ABS(open_amount) <= ABS(amount));

ALTER TABLE clearing_items ADD CONSTRAINT chk_cleared_amount_positive 
    CHECK (cleared_amount > 0);

-- 添加注释
COMMENT ON TABLE business_partners IS '业务伙伴主数据表';
COMMENT ON TABLE customers IS '客户主数据表';
COMMENT ON TABLE suppliers IS '供应商主数据表';
COMMENT ON TABLE open_items IS '未清项表（应收/应付行项目）';
COMMENT ON TABLE clearing_history IS '清账历史表';
COMMENT ON TABLE credit_checks IS '信用检查记录表';
COMMENT ON TABLE payment_proposals IS '付款建议表';
COMMENT ON TABLE aging_snapshots IS '账龄分析快照表';
COMMENT ON TABLE dunning_history IS '催款记录表';

COMMENT ON VIEW v_partner_overview IS '合并的伙伴信息视图（客户+供应商）';
COMMENT ON VIEW v_open_items_summary IS '未清项汇总视图';
COMMENT ON VIEW v_aging_analysis IS '账龄分析实时视图';
COMMENT ON VIEW v_customer_credit_exposure IS '客户信用额度使用情况视图';
