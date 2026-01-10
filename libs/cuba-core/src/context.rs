use serde::{Deserialize, Serialize};

/// Request context containing metadata about the current request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub trace_id: String,
    pub user_id: Option<String>,
    pub tenant_id: Option<String>,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            user_id: None,
            tenant_id: None,
        }
    }
}
