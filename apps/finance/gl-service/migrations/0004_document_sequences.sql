-- ============================================================================
-- GL Service: Document Number Sequences
-- 描述: 管理公司代码和会计年度的凭证编号范围
-- ============================================================================

CREATE TABLE document_sequences (
    company_code VARCHAR(4) NOT NULL,
    fiscal_year INT NOT NULL,
    document_type VARCHAR(2) NOT NULL, -- 如 SA, KR, DZ 等
    current_value BIGINT NOT NULL DEFAULT 0,
    prefix VARCHAR(2) DEFAULT '',
    
    PRIMARY KEY (company_code, fiscal_year, document_type)
);

-- 初始化一些示例序列号范围
INSERT INTO document_sequences (company_code, fiscal_year, document_type, current_value)
VALUES 
('1000', 2026, 'SA', 1000000000),
('1000', 2026, 'KR', 1900000000),
('1000', 2026, 'DZ', 1400000000);
