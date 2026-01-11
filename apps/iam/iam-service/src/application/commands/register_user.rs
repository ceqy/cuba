use serde::{Deserialize, Serialize};
use cuba_cqrs::Command;

#[derive(Debug, Deserialize, Serialize)]
pub struct RegisterUserCommand {
    pub username: String,
    pub email: String,
    pub password: String,
    pub tenant_id: Option<String>,
}

impl Command for RegisterUserCommand {
}
