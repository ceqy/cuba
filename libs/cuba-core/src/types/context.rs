use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub request_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub roles: Vec<String>,
    pub trace_id: String,
    pub locale: String,
}

impl RequestContext {
    pub fn new(
        request_id: impl Into<String>,
        tenant_id: impl Into<String>,
        user_id: impl Into<String>,
    ) -> Self {
        Self {
            request_id: request_id.into(),
            tenant_id: tenant_id.into(),
            user_id: user_id.into(),
            roles: Vec::new(),
            trace_id: String::new(),
            locale: "en-US".to_string(),
        }
    }
}
