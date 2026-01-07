use crate::domain::repositories::{RefreshTokenData, RefreshTokenRepository, UserRepository};
use crate::domain::services::{RefreshTokenClaims, TokenClaims, TokenError, TokenService};
use crate::domain::value_objects::UserId;
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::sync::Arc;

// ============================================================================
// JWT Service Implementation
// ============================================================================

/// JWT 服务配置
#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub access_token_expiry_minutes: i64,
    pub refresh_token_expiry_days: i64,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: "your-256-bit-secret-key-here-change-in-production".to_string(),
            issuer: "cuba-auth-service".to_string(),
            access_token_expiry_minutes: 15,
            refresh_token_expiry_days: 7,
        }
    }
}

/// JWT 服务实现
pub struct JwtService {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    user_repo: Arc<dyn UserRepository>,
    refresh_token_repo: Arc<dyn RefreshTokenRepository>,
}

impl JwtService {
    pub fn new(
        config: JwtConfig,
        user_repo: Arc<dyn UserRepository>,
        refresh_token_repo: Arc<dyn RefreshTokenRepository>,
    ) -> Self {
        let encoding_key = EncodingKey::from_secret(config.secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(config.secret.as_bytes());

        Self {
            config,
            encoding_key,
            decoding_key,
            user_repo,
            refresh_token_repo,
        }
    }

    fn generate_access_token_internal(
        &self,
        user_id: &str,
        tenant_id: &str,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<(String, i64), TokenError> {
        let now = Utc::now();
        let expires_in = self.config.access_token_expiry_minutes * 60;
        let exp = now + Duration::minutes(self.config.access_token_expiry_minutes);

        let claims = TokenClaims {
            sub: user_id.to_string(),
            tid: tenant_id.to_string(),
            iss: self.config.issuer.clone(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            roles: roles.clone(),
            permissions,
            token_type: if roles.contains(&"temp_2fa".to_string()) { "temp_2fa".to_string() } else { "access".to_string() },
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|e| TokenError::EncodingError(e.to_string()))?;

        Ok((token, expires_in))
    }

    fn generate_refresh_token_internal(&self, user_id: &str) -> Result<(String, String, chrono::DateTime<Utc>), TokenError> {
        let now = Utc::now();
        let expires_at = now + Duration::days(self.config.refresh_token_expiry_days);
        let jti = uuid::Uuid::new_v4().to_string();

        let claims = RefreshTokenClaims {
            sub: user_id.to_string(),
            exp: expires_at.timestamp(),
            iat: now.timestamp(),
            token_type: "refresh".to_string(),
            jti: jti.clone(),
        };

        let token = encode(&Header::new(Algorithm::HS256), &claims, &self.encoding_key)
            .map_err(|e| TokenError::EncodingError(e.to_string()))?;
            
        Ok((token, jti, expires_at))
    }

    fn decode_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, TokenError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.validate_aud = false;
        validation.set_required_spec_claims(&["sub", "exp"]);

        let token_data = decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                _ => TokenError::DecodingError(e.to_string()),
            })?;

        if token_data.claims.token_type != "refresh" {
            return Err(TokenError::TypeMismatch);
        }

        Ok(token_data.claims)
    }
}

#[async_trait::async_trait]
impl TokenService for JwtService {
    async fn generate_tokens(
        &self,
        user_id: String,
        tenant_id: String,
        roles: Vec<String>,
        permissions: Vec<String>,
    ) -> Result<(String, String, i64), TokenError> {
        let (access_token, expires_in) =
            self.generate_access_token_internal(&user_id, &tenant_id, roles, permissions)?;
        let (refresh_token, jti, expires_at) = self.generate_refresh_token_internal(&user_id)?;

        // 持久化 Refresh Token
        let token_data = RefreshTokenData {
            id: jti,
            user_id: UserId::parse(&user_id).map_err(|e| TokenError::Invalid)?,
            token_hash: refresh_token.clone(), // 实际生产环境应存储哈希，这里简化
            expires_at,
            is_revoked: false,
            created_at: Utc::now(),
        };

        self.refresh_token_repo.save(&token_data).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?;

        Ok((access_token, refresh_token, expires_in))
    }

    fn validate_token(&self, token: &str) -> Result<TokenClaims, TokenError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[&self.config.issuer]);
        validation.validate_aud = false;

        let token_data = decode::<TokenClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::Expired,
                _ => TokenError::DecodingError(e.to_string()),
            })?;

        if token_data.claims.token_type != "access" && token_data.claims.token_type != "temp_2fa" {
            return Err(TokenError::TypeMismatch);
        }

        Ok(token_data.claims)
    }

    async fn refresh_tokens(&self, refresh_token: &str) -> Result<(String, String, i64), TokenError> {
        let claims = self.decode_refresh_token(refresh_token)?;

        // 检查数据库中是否存在且未撤销
        let token_data = self.refresh_token_repo.find_by_hash(refresh_token).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?
            .ok_or(TokenError::Invalid)?;

        if token_data.is_revoked {
            return Err(TokenError::Revoked);
        }

        // 检查用户是否仍然有效
        let user = self.user_repo.find_by_id(&token_data.user_id).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?
            .ok_or(TokenError::Invalid)?;

        if !user.is_active() {
            return Err(TokenError::Invalid);
        }

        // 获取最新的角色和权限
        let permissions = self.user_repo.get_user_permissions(user.id()).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?;
        let permission_strings: Vec<String> = permissions.iter().map(|p| p.to_string()).collect();
        let role_strings: Vec<String> = user.roles().iter().map(|r| r.to_string()).collect();

        // 撤销旧 Token
        self.refresh_token_repo.revoke(refresh_token).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?;

        // 生成新 Token
        // FIXME: 这里的 tenant_id 需要从数据库获取或从 claims 中传递，目前 user 没有直接存储 tenant_id
        self.generate_tokens(user.id().to_string(), String::new(), role_strings, permission_strings).await
    }

    async fn revoke_token(&self, refresh_token: &str) -> Result<(), TokenError> {
        self.refresh_token_repo.revoke(refresh_token).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?;
        Ok(())
    }

    async fn revoke_all_for_user(&self, user_id: &String) -> Result<(), TokenError> {
        let uid = UserId::parse(user_id).map_err(|e| TokenError::Invalid)?;
        self.refresh_token_repo.revoke_all_for_user(&uid).await
            .map_err(|e| TokenError::InfrastructureError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aggregates::User;
    use crate::domain::repositories::RepositoryError; // Added import
    use crate::domain::value_objects::{Permission, RoleId};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // Mock UserRepository
    struct MockUserRepository;
    
    #[async_trait]
    impl UserRepository for MockUserRepository {
        async fn save(&self, _user: &mut User) -> Result<(), RepositoryError> { Ok(()) }
        async fn find_by_id(&self, _id: &UserId) -> Result<Option<User>, RepositoryError> { 
            // Return a dummy user for refresh_tokens test
            // We need to construct a minimal user. Since User fields are private, we might need a constructor or accessors.
            // But verify_token doesn't hit DB. refresh_tokens does.
            // Let's rely on integration tests for complex flows if unit test is too hard due to private fields.
            // OR use User::register to create a dummy user.
            let mut user = User::register("test".to_string(), "test@test.com".to_string(), "pass").unwrap();
            
            // For refresh_tokens test, we need is_active=true (default)
            // And roles/permissions.
            Ok(Some(user))
        }
        async fn find_by_username(&self, _username: &str) -> Result<Option<User>, RepositoryError> { Ok(None) }
        async fn find_by_email(&self, _email: &str) -> Result<Option<User>, RepositoryError> { Ok(None) }
        async fn username_exists(&self, _username: &str) -> Result<bool, RepositoryError> { Ok(false) }
        async fn email_exists(&self, _email: &str) -> Result<bool, RepositoryError> { Ok(false) }
        async fn get_user_permissions(&self, _id: &UserId) -> Result<Vec<Permission>, RepositoryError> { Ok(vec![]) }
        async fn delete(&self, _id: &UserId) -> Result<(), RepositoryError> { Ok(()) }
        async fn find_all(&self, _search: Option<&str>, _role: Option<&RoleId>, _limit: i64, _offset: i64) -> Result<Vec<User>, RepositoryError> { Ok(vec![]) }
        async fn count_all(&self, _search: Option<&str>, _role: Option<&RoleId>) -> Result<i64, RepositoryError> { Ok(0) }
    }

    // Mock RefreshTokenRepository
    struct MockRefreshTokenRepository {
        tokens: Mutex<HashMap<String, RefreshTokenData>>,
    }
    
    impl MockRefreshTokenRepository {
        fn new() -> Self {
            Self { tokens: Mutex::new(HashMap::new()) }
        }
    }

    #[async_trait]
    impl RefreshTokenRepository for MockRefreshTokenRepository {
        async fn save(&self, token: &RefreshTokenData) -> Result<(), RepositoryError> {
            self.tokens.lock().unwrap().insert(token.token_hash.clone(), token.clone());
            Ok(())
        }
        async fn find_by_hash(&self, hash: &str) -> Result<Option<RefreshTokenData>, RepositoryError> {
            Ok(self.tokens.lock().unwrap().get(hash).cloned())
        }
        async fn revoke(&self, hash: &str) -> Result<(), RepositoryError> {
            if let Some(token) = self.tokens.lock().unwrap().get_mut(hash) {
                token.is_revoked = true;
            }
            Ok(())
        }
        async fn revoke_all_for_user(&self, _user_id: &UserId) -> Result<(), RepositoryError> { Ok(()) }
        async fn cleanup_expired(&self) -> Result<u64, RepositoryError> { Ok(0) }
    }

    #[tokio::test]
    async fn test_generate_and_validate_token() {
        let service = JwtService::new(
            JwtConfig::default(), 
            Arc::new(MockUserRepository), 
            Arc::new(MockRefreshTokenRepository::new())
        );

        let (access_token, _, _) = service
            .generate_tokens(
                "user-123".to_string(),
                "tenant-123".to_string(),
                vec!["admin".to_string()],
                vec!["sales_order:create".to_string()],
            )
            .await
            .unwrap();

        let claims = service.validate_token(&access_token).unwrap();
        assert_eq!(claims.sub, "user-123");
        assert_eq!(claims.tid, "tenant-123");
        assert_eq!(claims.roles, vec!["admin"]);
        assert_eq!(claims.permissions, vec!["sales_order:create"]);
    }

    #[tokio::test]
    async fn test_refresh_tokens() {
        let service = JwtService::new(
            JwtConfig::default(), 
            Arc::new(MockUserRepository), 
            Arc::new(MockRefreshTokenRepository::new())
        );

        let (_, refresh_token, _) = service
            .generate_tokens("user-123".to_string(), "default".to_string(), vec![], vec![])
            .await
            .unwrap();

        // Need to make sure find_by_hash returns something in test_refresh_tokens.
        // It relies on save() being called in generate_tokens, which we implemented in mock.
        
        // Also need find_by_id in UserRepository to return user. MockUserRepository does return a dummy user.

        let (new_access, _, _) = service.refresh_tokens(&refresh_token).await.unwrap();
        assert!(!new_access.is_empty());
    }
}
