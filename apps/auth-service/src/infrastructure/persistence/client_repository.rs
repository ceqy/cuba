//! PostgreSQL Client Repository implementation

use crate::domain::repositories::{ClientRepository, ClientData, RepositoryError};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;

pub struct PgClientRepository {
    pool: Arc<PgPool>,
}

impl PgClientRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ClientRepository for PgClientRepository {
    async fn save(&self, client: &ClientData) -> Result<(), RepositoryError> {
        sqlx::query(
            r#"
            INSERT INTO oauth2_clients (client_id, client_secret_hash, name, redirect_uris, grant_types, scopes, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (client_id) DO UPDATE SET
                client_secret_hash = EXCLUDED.client_secret_hash,
                name = EXCLUDED.name,
                redirect_uris = EXCLUDED.redirect_uris,
                grant_types = EXCLUDED.grant_types,
                scopes = EXCLUDED.scopes
            "#,
        )
        .bind(&client.client_id)
        .bind(&client.client_secret)
        .bind(&client.name)
        .bind(&client.redirect_uris)
        .bind(&client.grant_types)
        .bind(&client.scopes)
        .bind(client.created_at)
        .execute(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn find_by_id(&self, client_id: &str) -> Result<Option<ClientData>, RepositoryError> {
        let row = sqlx::query(
            "SELECT client_id, client_secret_hash, name, redirect_uris, grant_types, scopes, created_at FROM oauth2_clients WHERE client_id = $1",
        )
        .bind(client_id)
        .fetch_optional(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        match row {
            Some(row) => {
                Ok(Some(ClientData {
                    client_id: row.get("client_id"),
                    client_secret: row.get("client_secret_hash"),
                    name: row.get("name"),
                    redirect_uris: row.get("redirect_uris"),
                    grant_types: row.get("grant_types"),
                    scopes: row.get("scopes"),
                    created_at: row.get("created_at"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn find_all(&self, limit: i64, offset: i64) -> Result<Vec<ClientData>, RepositoryError> {
        let rows = sqlx::query(
            "SELECT client_id, client_secret_hash, name, redirect_uris, grant_types, scopes, created_at FROM oauth2_clients LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool.as_ref())
        .await
        .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;

        let mut clients = Vec::new();
        for row in rows {
            clients.push(ClientData {
                client_id: row.get("client_id"),
                client_secret: row.get("client_secret_hash"),
                name: row.get("name"),
                redirect_uris: row.get("redirect_uris"),
                grant_types: row.get("grant_types"),
                scopes: row.get("scopes"),
                created_at: row.get("created_at"),
            });
        }
        Ok(clients)
    }

    async fn count_all(&self) -> Result<i64, RepositoryError> {
        let row = sqlx::query("SELECT COUNT(*) FROM oauth2_clients")
            .fetch_one(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
            
        Ok(row.get::<i64, _>(0))
    }

    async fn delete(&self, client_id: &str) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM oauth2_clients WHERE client_id = $1")
            .bind(client_id)
            .execute(self.pool.as_ref())
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}
