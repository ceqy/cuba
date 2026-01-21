use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StreamingParams {
    pub batch_size: usize,
    pub timeout_seconds: u64,
}

impl Default for StreamingParams {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            timeout_seconds: 300,
        }
    }
}
