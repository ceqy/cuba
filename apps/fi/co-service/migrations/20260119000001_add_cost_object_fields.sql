-- Add minimal cost object fields for CO
-- Date: 2026-01-19

-- =============================================================================
-- Allocation senders
-- =============================================================================
ALTER TABLE allocation_senders
ADD COLUMN IF NOT EXISTS cost_center VARCHAR(10),
ADD COLUMN IF NOT EXISTS profit_center VARCHAR(10),
ADD COLUMN IF NOT EXISTS segment VARCHAR(10),
ADD COLUMN IF NOT EXISTS internal_order VARCHAR(12),
ADD COLUMN IF NOT EXISTS wbs_element VARCHAR(24);

COMMENT ON COLUMN allocation_senders.cost_center IS 'KOSTL 成本中心';
COMMENT ON COLUMN allocation_senders.profit_center IS 'PRCTR 利润中心';
COMMENT ON COLUMN allocation_senders.segment IS 'SEGMENT 段';
COMMENT ON COLUMN allocation_senders.internal_order IS 'AUFNR 内部订单';
COMMENT ON COLUMN allocation_senders.wbs_element IS 'PS_PSP_PNR WBS 元素';

-- =============================================================================
-- Allocation receivers
-- =============================================================================
ALTER TABLE allocation_receivers
ADD COLUMN IF NOT EXISTS cost_center VARCHAR(10),
ADD COLUMN IF NOT EXISTS profit_center VARCHAR(10),
ADD COLUMN IF NOT EXISTS segment VARCHAR(10),
ADD COLUMN IF NOT EXISTS internal_order VARCHAR(12),
ADD COLUMN IF NOT EXISTS wbs_element VARCHAR(24);

COMMENT ON COLUMN allocation_receivers.cost_center IS 'KOSTL 成本中心';
COMMENT ON COLUMN allocation_receivers.profit_center IS 'PRCTR 利润中心';
COMMENT ON COLUMN allocation_receivers.segment IS 'SEGMENT 段';
COMMENT ON COLUMN allocation_receivers.internal_order IS 'AUFNR 内部订单';
COMMENT ON COLUMN allocation_receivers.wbs_element IS 'PS_PSP_PNR WBS 元素';
