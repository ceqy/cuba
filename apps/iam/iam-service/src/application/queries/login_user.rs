use serde::{Deserialize, Serialize};
use cuba_cqrs::Query;

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginUserQuery {
    pub username: String,
    pub password: String,
    pub tenant_id: Option<String>,
}

impl Query for LoginUserQuery {}
