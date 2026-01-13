use async_trait::async_trait;
use crate::domain::entities::{OAuthClient, AuthorizationCode, RefreshToken};
use crate::domain::repositories::{ClientRepository, AuthCodeRepository, RefreshTokenRepository};
use cuba_database::DbPool;
use sqlx::Row;

pub struct PostgresOAuthRepository {
    pool: DbPool,
}

impl PostgresOAuthRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientRepository for PostgresOAuthRepository {
    async fn find_by_id(&self, client_id: &str) -> Result<Option<OAuthClient>, anyhow::Error> {
        let row = sqlx::query("SELECT * FROM oauth_clients WHERE client_id = $1")
            .bind(client_id)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|r| OAuthClient {
            client_id: r.try_get("client_id").unwrap(),
            client_secret: r.try_get("client_secret").unwrap(),
            name: r.try_get("name").unwrap(),
            redirect_uris: r.try_get("redirect_uris").unwrap_or_default(),
            grant_types: r.try_get("grant_types").unwrap_or_default(),
            scopes: r.try_get("scopes").unwrap_or_default(),
            created_at: r.try_get("created_at").unwrap(),
        }))
    }

    async fn save(&self, client: &OAuthClient) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO oauth_clients (client_id, client_secret, name, redirect_uris, grant_types, scopes, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT (client_id) DO UPDATE SET 
                client_secret = EXCLUDED.client_secret,
                name = EXCLUDED.name,
                redirect_uris = EXCLUDED.redirect_uris,
                grant_types = EXCLUDED.grant_types,
                scopes = EXCLUDED.scopes"
        )
        .bind(&client.client_id)
        .bind(&client.client_secret)
        .bind(&client.name)
        .bind(&client.redirect_uris)
        .bind(&client.grant_types)
        .bind(&client.scopes)
        .bind(client.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, client_id: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM oauth_clients WHERE client_id = $1")
            .bind(client_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<OAuthClient>, anyhow::Error> {
        let rows = sqlx::query("SELECT * FROM oauth_clients")
            .fetch_all(&self.pool)
            .await?;
            
        let mut clients = Vec::new();
        for r in rows {
            clients.push(OAuthClient {
                client_id: r.try_get("client_id")?,
                client_secret: r.try_get("client_secret")?,
                name: r.try_get("name")?,
                redirect_uris: r.try_get("redirect_uris").unwrap_or_default(),
                grant_types: r.try_get("grant_types").unwrap_or_default(),
                scopes: r.try_get("scopes").unwrap_or_default(),
                created_at: r.try_get("created_at")?,
            });
        }
        Ok(clients)
    }
}

#[async_trait]
impl AuthCodeRepository for PostgresOAuthRepository {
    async fn find_by_code(&self, code: &str) -> Result<Option<AuthorizationCode>, anyhow::Error> {
        let row = sqlx::query("SELECT * FROM oauth_authorization_codes WHERE code = $1 AND expires_at > NOW()")
            .bind(code)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|r| AuthorizationCode {
            code: r.try_get("code").unwrap(),
            client_id: r.try_get("client_id").unwrap(),
            user_id: r.try_get("user_id").unwrap(),
            redirect_uri: r.try_get("redirect_uri").unwrap(),
            scope: r.try_get("scope").unwrap(),
            code_challenge: r.try_get("code_challenge").ok(),
            code_challenge_method: r.try_get("code_challenge_method").ok(),
            expires_at: r.try_get("expires_at").unwrap(),
            created_at: r.try_get("created_at").unwrap(),
        }))
    }

    async fn save(&self, code: &AuthorizationCode) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO oauth_authorization_codes (code, client_id, user_id, redirect_uri, scope, code_challenge, code_challenge_method, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
        )
        .bind(&code.code)
        .bind(&code.client_id)
        .bind(&code.user_id)
        .bind(&code.redirect_uri)
        .bind(&code.scope)
        .bind(&code.code_challenge)
        .bind(&code.code_challenge_method)
        .bind(code.expires_at)
        .bind(code.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, code: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM oauth_authorization_codes WHERE code = $1")
            .bind(code)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl RefreshTokenRepository for PostgresOAuthRepository {
    async fn find_by_token(&self, token: &str) -> Result<Option<RefreshToken>, anyhow::Error> {
        let row = sqlx::query("SELECT * FROM oauth_refresh_tokens WHERE token = $1 AND expires_at > NOW()")
            .bind(token)
            .fetch_optional(&self.pool)
            .await?;
            
        Ok(row.map(|r| RefreshToken {
            token: r.try_get("token").unwrap(),
            client_id: r.try_get("client_id").unwrap(),
            user_id: r.try_get("user_id").unwrap(),
            scope: r.try_get("scope").unwrap(),
            expires_at: r.try_get("expires_at").unwrap(),
            created_at: r.try_get("created_at").unwrap(),
        }))
    }

    async fn save(&self, token: &RefreshToken) -> Result<(), anyhow::Error> {
        sqlx::query(
            "INSERT INTO oauth_refresh_tokens (token, client_id, user_id, scope, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)"
        )
        .bind(&token.token)
        .bind(&token.client_id)
        .bind(&token.user_id)
        .bind(&token.scope)
        .bind(token.expires_at)
        .bind(token.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, token: &str) -> Result<(), anyhow::Error> {
        sqlx::query("DELETE FROM oauth_refresh_tokens WHERE token = $1")
            .bind(token)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
