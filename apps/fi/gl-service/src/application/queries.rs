use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct GetJournalEntryQuery {
    pub id: Uuid,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ListJournalEntriesQuery {
    pub company_code: String,
    pub status: Option<String>,
    pub page: u64,
    pub page_size: u64,
}

/// 按特殊总账类型查询
#[derive(Debug, Deserialize, Clone)]
pub struct ListSpecialGlEntriesQuery {
    pub company_code: String,
    pub special_gl_type: String,  // SAP 代码: A, F, V, W
    pub status: Option<String>,
    pub page: u64,
    pub page_size: u64,
}

/// 按业务伙伴和特殊总账类型查询
#[derive(Debug, Deserialize, Clone)]
pub struct ListBusinessPartnerSpecialGlQuery {
    pub company_code: String,
    pub business_partner: String,
    pub special_gl_type: Option<String>,  // 可选，不指定则返回所有特殊总账
    pub status: Option<String>,
    pub page: u64,
    pub page_size: u64,
}
