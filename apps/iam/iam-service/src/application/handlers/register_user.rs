use async_trait::async_trait;
use cuba_cqrs::CommandHandler;
use crate::application::commands::RegisterUserCommand;
use crate::domain::aggregates::User;
use crate::domain::repositories::UserRepository;
use anyhow::{Result, bail};
use std::sync::Arc;

pub struct RegisterUserHandler {
    user_repo: Arc<dyn UserRepository>,
}

impl RegisterUserHandler {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self { user_repo }
    }
}

#[async_trait]
impl CommandHandler<RegisterUserCommand> for RegisterUserHandler {
    type Output = User;
    
    async fn handle(&self, cmd: RegisterUserCommand) -> Result<User> {
        // 1. Check if user exists
        if self.user_repo.find_by_username(&cmd.username).await?.is_some() {
            bail!("Username already exists");
        }
        if self.user_repo.find_by_email(&cmd.email).await?.is_some() {
            bail!("Email already exists");
        }

        // 2. Hash password (TODO: use bcrypt service)
        let password_hash = format!("hashed_{}", cmd.password);

        // 3. Create user
        let user = User::new(
            cmd.username,
            cmd.email,
            password_hash,
            cmd.tenant_id,
        );

        // 4. Save
        self.user_repo.save(&user).await?;

        Ok(user)
    }
}
