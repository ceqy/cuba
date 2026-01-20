# CUBA ERP 现代认证系统设计方案

## 目录

1. [概述](#概述)
2. [架构设计](#架构设计)
3. [实施阶段](#实施阶段)
4. [第一阶段：JWT 增强](#第一阶段jwt-增强)
5. [第二阶段：Passkey/WebAuthn](#第二阶段passkeywebauthn)
6. [第三阶段：OAuth 社交登录](#第三阶段oauth-社交登录)
7. [第四阶段：安全增强](#第四阶段安全增强)
8. [数据库设计](#数据库设计)
9. [API 设计](#api-设计)
10. [技术栈](#技术栈)
11. [迁移计划](#迁移计划)

---

## 概述

### 背景

当前系统使用基础 JWT 认证，存在以下问题：
- Token 无法主动撤销
- 缺少无密码登录支持
- 没有社交登录功能
- 安全检测能力不足

### 目标

构建一个现代化的认证系统，具备：
- ✅ 可撤销的 Token 管理
- ✅ Passkey/WebAuthn 无密码登录
- ✅ OAuth 2.0 社交登录
- ✅ 异常检测和安全防护
- ✅ 多租户支持

### 优势

| 特性 | 当前系统 | 新系统 |
|------|---------|--------|
| Token 撤销 | ❌ 不支持 | ✅ 实时撤销 |
| 无密码登录 | ❌ 不支持 | ✅ Passkey |
| 社交登录 | ❌ 不支持 | ✅ Google/GitHub/微信 |
| 异常检测 | ❌ 不支持 | ✅ 设备指纹+行为分析 |
| 多因素认证 | ❌ 不支持 | ✅ TOTP/SMS |

---

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         客户端层                                  │
├─────────────────────────────────────────────────────────────────┤
│  Web App    │   Mobile App   │   Desktop App   │   API Client   │
└──────┬──────┴───────┬────────┴────────┬────────┴───────┬────────┘
       │              │                 │                │
       └──────────────┴────────┬────────┴────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                       API Gateway (Envoy)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │ Rate Limit  │  │ Auth Filter │  │ gRPC-JSON Transcoder    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘  │
└─────────────────────────────┬───────────────────────────────────┘
                               │
       ┌───────────────────────┼───────────────────────┐
       │                       │                       │
       ▼                       ▼                       ▼
┌─────────────┐      ┌─────────────────┐      ┌─────────────┐
│ Auth Service│      │ Security Service│      │ RBAC Service│
│             │◄────►│                 │◄────►│             │
│ - Login     │      │ - Rate Limit   │      │ - Roles     │
│ - Register  │      │ - Device FP    │      │ - Perms     │
│ - Token     │      │ - Anomaly Det  │      │ - Policies  │
│ - Passkey   │      │ - IP Analysis  │      │             │
│ - OAuth     │      │                 │      │             │
└──────┬──────┘      └────────┬────────┘      └─────────────┘
       │                      │
       ▼                      ▼
┌─────────────────────────────────────────────────────────────────┐
│                         数据存储层                                │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   PostgreSQL    │      Redis      │      ClickHouse             │
│   (主数据)      │   (缓存/会话)    │    (审计日志)               │
└─────────────────┴─────────────────┴─────────────────────────────┘
```

### 组件说明

| 组件 | 职责 | 技术 |
|------|------|------|
| Auth Service | 核心认证逻辑 | Rust + Tonic |
| Security Service | 安全检测和防护 | Rust + ML |
| RBAC Service | 权限管理 | Rust + Tonic |
| PostgreSQL | 用户数据、凭证 | PostgreSQL 15+ |
| Redis | Token 黑名单、会话 | Redis 7+ |
| ClickHouse | 审计日志分析 | ClickHouse |

---

## 实施阶段

### 阶段规划

```
第一阶段 ──────► 第二阶段 ──────► 第三阶段 ──────► 第四阶段
JWT 增强        Passkey         OAuth 集成       安全增强
(基础)          (无密码)        (社交登录)       (高级防护)
```

### 各阶段重点

| 阶段 | 核心功能 | 依赖 |
|------|---------|------|
| 第一阶段 | JTI + Token 撤销 + 黑名单 | Redis |
| 第二阶段 | WebAuthn + Passkey | 浏览器支持 |
| 第三阶段 | Google/GitHub/微信登录 | OAuth 配置 |
| 第四阶段 | 设备指纹 + 异常检测 | ML 模型 |

---

## 第一阶段：JWT 增强

### 1.1 Token 结构改进

**当前 JWT Payload:**
```json
{
  "sub": "user_id",
  "exp": 1234567890,
  "iat": 1234567890
}
```

**改进后 JWT Payload:**
```json
{
  "jti": "unique-token-id",      // Token ID，用于撤销
  "sub": "user_id",
  "tenant_id": "tenant_id",
  "device_id": "device_fingerprint",
  "session_id": "session_id",
  "exp": 1234567890,
  "iat": 1234567890,
  "nbf": 1234567890,            // Not Before
  "iss": "cuba-auth",           // Issuer
  "aud": ["cuba-api"]           // Audience
}
```

### 1.2 Token Manager 实现

```rust
// src/auth/token_manager.rs

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Token 声明结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    /// Token 唯一标识符 (用于撤销)
    pub jti: String,
    /// 用户 ID
    pub sub: String,
    /// 租户 ID
    pub tenant_id: String,
    /// 设备 ID
    pub device_id: Option<String>,
    /// 会话 ID
    pub session_id: String,
    /// 过期时间
    pub exp: i64,
    /// 签发时间
    pub iat: i64,
    /// 生效时间
    pub nbf: i64,
    /// 签发者
    pub iss: String,
    /// 受众
    pub aud: Vec<String>,
}

/// Token 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Access,
    Refresh,
}

/// Token 管理器
pub struct TokenManager {
    /// JWT 编码密钥
    encoding_key: EncodingKey,
    /// JWT 解码密钥
    decoding_key: DecodingKey,
    /// Redis 连接池
    redis_pool: redis::Client,
    /// Access Token 有效期
    access_token_ttl: Duration,
    /// Refresh Token 有效期
    refresh_token_ttl: Duration,
    /// 签发者
    issuer: String,
}

impl TokenManager {
    /// 创建新的 Token Manager
    pub fn new(
        secret: &str,
        redis_url: &str,
        access_ttl_minutes: i64,
        refresh_ttl_days: i64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            redis_pool: redis::Client::open(redis_url)?,
            access_token_ttl: Duration::minutes(access_ttl_minutes),
            refresh_token_ttl: Duration::days(refresh_ttl_days),
            issuer: "cuba-auth".to_string(),
        })
    }

    /// 生成 Token 对 (Access + Refresh)
    pub async fn generate_token_pair(
        &self,
        user_id: &str,
        tenant_id: &str,
        device_id: Option<&str>,
    ) -> Result<TokenPair, TokenError> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // 生成 Access Token
        let access_claims = TokenClaims {
            jti: Uuid::new_v4().to_string(),
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            device_id: device_id.map(String::from),
            session_id: session_id.clone(),
            exp: (now + self.access_token_ttl).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            iss: self.issuer.clone(),
            aud: vec!["cuba-api".to_string()],
        };

        let access_token = encode(
            &Header::default(),
            &access_claims,
            &self.encoding_key,
        ).map_err(|e| TokenError::EncodingFailed(e.to_string()))?;

        // 生成 Refresh Token (使用 UUID，存储在 Redis)
        let refresh_token = Uuid::new_v4().to_string();

        // 存储 Refresh Token 到 Redis
        self.store_refresh_token(
            &refresh_token,
            user_id,
            tenant_id,
            &session_id,
            device_id,
        ).await?;

        // 存储会话信息
        self.store_session(&session_id, user_id, tenant_id, device_id).await?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: self.access_token_ttl.num_seconds(),
            refresh_expires_in: self.refresh_token_ttl.num_seconds(),
        })
    }

    /// 验证 Access Token
    pub async fn verify_access_token(
        &self,
        token: &str,
    ) -> Result<TokenClaims, TokenError> {
        // 解码 Token
        let mut validation = Validation::default();
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&["cuba-api"]);

        let token_data = decode::<TokenClaims>(
            token,
            &self.decoding_key,
            &validation,
        ).map_err(|e| TokenError::InvalidToken(e.to_string()))?;

        let claims = token_data.claims;

        // 检查 Token 是否在黑名单中
        if self.is_token_blacklisted(&claims.jti).await? {
            return Err(TokenError::TokenRevoked);
        }

        // 检查会话是否有效
        if !self.is_session_valid(&claims.session_id).await? {
            return Err(TokenError::SessionInvalid);
        }

        Ok(claims)
    }

    /// 刷新 Token
    pub async fn refresh_tokens(
        &self,
        refresh_token: &str,
    ) -> Result<TokenPair, TokenError> {
        // 获取 Refresh Token 数据
        let refresh_data = self.get_refresh_token_data(refresh_token).await?
            .ok_or(TokenError::RefreshTokenInvalid)?;

        // 删除旧的 Refresh Token (一次性使用)
        self.delete_refresh_token(refresh_token).await?;

        // 生成新的 Token 对
        self.generate_token_pair(
            &refresh_data.user_id,
            &refresh_data.tenant_id,
            refresh_data.device_id.as_deref(),
        ).await
    }

    /// 撤销 Token
    pub async fn revoke_token(&self, jti: &str, exp: i64) -> Result<(), TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        let ttl = exp - Utc::now().timestamp();
        if ttl > 0 {
            let key = format!("token:blacklist:{}", jti);
            conn.set_ex::<_, _, ()>(&key, "revoked", ttl as u64).await
                .map_err(|e| TokenError::StorageError(e.to_string()))?;
        }

        Ok(())
    }

    /// 撤销用户所有 Token
    pub async fn revoke_all_user_tokens(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<(), TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        // 获取用户所有会话
        let sessions_key = format!("user:{}:{}:sessions", tenant_id, user_id);
        let sessions: Vec<String> = conn.smembers(&sessions_key).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        // 删除所有会话
        for session_id in sessions {
            let session_key = format!("session:{}", session_id);
            conn.del::<_, ()>(&session_key).await
                .map_err(|e| TokenError::StorageError(e.to_string()))?;
        }

        // 清空会话列表
        conn.del::<_, ()>(&sessions_key).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// 检查 Token 是否在黑名单中
    async fn is_token_blacklisted(&self, jti: &str) -> Result<bool, TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        let key = format!("token:blacklist:{}", jti);
        let exists: bool = conn.exists(&key).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        Ok(exists)
    }

    /// 检查会话是否有效
    async fn is_session_valid(&self, session_id: &str) -> Result<bool, TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        let key = format!("session:{}", session_id);
        let exists: bool = conn.exists(&key).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        Ok(exists)
    }

    /// 存储 Refresh Token
    async fn store_refresh_token(
        &self,
        token: &str,
        user_id: &str,
        tenant_id: &str,
        session_id: &str,
        device_id: Option<&str>,
    ) -> Result<(), TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        let key = format!("refresh_token:{}", token);
        let data = RefreshTokenData {
            user_id: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            session_id: session_id.to_string(),
            device_id: device_id.map(String::from),
            created_at: Utc::now(),
        };

        let json = serde_json::to_string(&data)
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        conn.set_ex::<_, _, ()>(
            &key,
            json,
            self.refresh_token_ttl.num_seconds() as u64,
        ).await.map_err(|e| TokenError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// 获取 Refresh Token 数据
    async fn get_refresh_token_data(
        &self,
        token: &str,
    ) -> Result<Option<RefreshTokenData>, TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        let key = format!("refresh_token:{}", token);
        let json: Option<String> = conn.get(&key).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        match json {
            Some(j) => {
                let data: RefreshTokenData = serde_json::from_str(&j)
                    .map_err(|e| TokenError::StorageError(e.to_string()))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// 删除 Refresh Token
    async fn delete_refresh_token(&self, token: &str) -> Result<(), TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        let key = format!("refresh_token:{}", token);
        conn.del::<_, ()>(&key).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// 存储会话
    async fn store_session(
        &self,
        session_id: &str,
        user_id: &str,
        tenant_id: &str,
        device_id: Option<&str>,
    ) -> Result<(), TokenError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        // 存储会话数据
        let session_key = format!("session:{}", session_id);
        let session_data = SessionData {
            user_id: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            device_id: device_id.map(String::from),
            created_at: Utc::now(),
            last_activity: Utc::now(),
        };

        let json = serde_json::to_string(&session_data)
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        conn.set_ex::<_, _, ()>(
            &session_key,
            json,
            self.refresh_token_ttl.num_seconds() as u64,
        ).await.map_err(|e| TokenError::StorageError(e.to_string()))?;

        // 添加到用户会话列表
        let user_sessions_key = format!("user:{}:{}:sessions", tenant_id, user_id);
        conn.sadd::<_, _, ()>(&user_sessions_key, session_id).await
            .map_err(|e| TokenError::StorageError(e.to_string()))?;

        Ok(())
    }
}

/// Token 对
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_expires_in: i64,
}

/// Refresh Token 数据
#[derive(Debug, Serialize, Deserialize)]
struct RefreshTokenData {
    user_id: String,
    tenant_id: String,
    session_id: String,
    device_id: Option<String>,
    created_at: DateTime<Utc>,
}

/// 会话数据
#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    user_id: String,
    tenant_id: String,
    device_id: Option<String>,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}

/// Token 错误
#[derive(Debug, thiserror::Error)]
pub enum TokenError {
    #[error("Token encoding failed: {0}")]
    EncodingFailed(String),
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    #[error("Token has been revoked")]
    TokenRevoked,
    #[error("Session is invalid")]
    SessionInvalid,
    #[error("Refresh token is invalid or expired")]
    RefreshTokenInvalid,
    #[error("Storage error: {0}")]
    StorageError(String),
}
```

---

## 第二阶段：Passkey/WebAuthn

### 2.1 概述

Passkey 是基于 WebAuthn 标准的无密码认证方式，具有以下优点：
- 🔐 更安全：基于公钥加密，无法被钓鱼
- 🚀 更便捷：指纹/面部识别，无需记住密码
- 🌐 跨平台：支持手机、电脑、安全密钥

### 2.2 Passkey Manager 实现

```rust
// src/auth/passkey_manager.rs

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use webauthn_rs::prelude::*;

/// Passkey 凭证
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct PasskeyCredential {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub credential_id: Vec<u8>,
    pub public_key: Vec<u8>,
    pub counter: u32,
    pub aaguid: Option<Vec<u8>>,
    pub device_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// 注册请求响应
#[derive(Debug, Serialize, Deserialize)]
pub struct RegistrationChallenge {
    pub challenge_id: String,
    pub options: CreationChallengeResponse,
}

/// 认证请求响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthenticationChallenge {
    pub challenge_id: String,
    pub options: RequestChallengeResponse,
}

/// Passkey 管理器
pub struct PasskeyManager {
    webauthn: Webauthn,
    db_pool: PgPool,
    redis_pool: redis::Client,
}

impl PasskeyManager {
    /// 创建新的 Passkey Manager
    pub fn new(
        rp_id: &str,
        rp_origin: &str,
        rp_name: &str,
        db_pool: PgPool,
        redis_pool: redis::Client,
    ) -> Result<Self, PasskeyError> {
        let rp_origin = Url::parse(rp_origin)
            .map_err(|e| PasskeyError::ConfigError(e.to_string()))?;

        let builder = WebauthnBuilder::new(rp_id, &rp_origin)
            .map_err(|e| PasskeyError::ConfigError(e.to_string()))?
            .rp_name(rp_name);

        let webauthn = builder.build()
            .map_err(|e| PasskeyError::ConfigError(e.to_string()))?;

        Ok(Self {
            webauthn,
            db_pool,
            redis_pool,
        })
    }

    /// 开始 Passkey 注册
    pub async fn start_registration(
        &self,
        user_id: &str,
        user_name: &str,
        user_display_name: &str,
        tenant_id: &str,
    ) -> Result<RegistrationChallenge, PasskeyError> {
        // 获取用户现有凭证
        let existing_credentials = self.get_user_credentials(user_id, tenant_id).await?;
        let exclude_credentials: Vec<CredentialID> = existing_credentials
            .iter()
            .map(|c| CredentialID::from(c.credential_id.clone()))
            .collect();

        // 创建用户标识
        let user_unique_id = Uuid::parse_str(user_id)
            .map_err(|e| PasskeyError::InvalidUserId(e.to_string()))?;

        // 生成注册选项
        let (ccr, reg_state) = self.webauthn
            .start_passkey_registration(
                user_unique_id,
                user_name,
                user_display_name,
                Some(exclude_credentials),
            )
            .map_err(|e| PasskeyError::WebAuthnError(e.to_string()))?;

        // 存储注册状态到 Redis
        let challenge_id = Uuid::new_v4().to_string();
        self.store_registration_state(&challenge_id, user_id, tenant_id, &reg_state).await?;

        Ok(RegistrationChallenge {
            challenge_id,
            options: ccr,
        })
    }

    /// 完成 Passkey 注册
    pub async fn finish_registration(
        &self,
        challenge_id: &str,
        response: RegisterPublicKeyCredential,
        device_name: Option<&str>,
    ) -> Result<PasskeyCredential, PasskeyError> {
        // 获取注册状态
        let (user_id, tenant_id, reg_state) = self
            .get_registration_state(challenge_id)
            .await?
            .ok_or(PasskeyError::ChallengeExpired)?;

        // 验证注册响应
        let passkey = self.webauthn
            .finish_passkey_registration(&response, &reg_state)
            .map_err(|e| PasskeyError::WebAuthnError(e.to_string()))?;

        // 保存凭证到数据库
        let credential = self.save_credential(
            &user_id,
            &tenant_id,
            &passkey,
            device_name,
        ).await?;

        // 删除注册状态
        self.delete_registration_state(challenge_id).await?;

        Ok(credential)
    }

    /// 开始 Passkey 认证
    pub async fn start_authentication(
        &self,
        user_id: Option<&str>,
        tenant_id: &str,
    ) -> Result<AuthenticationChallenge, PasskeyError> {
        let allow_credentials = match user_id {
            Some(uid) => {
                let credentials = self.get_user_credentials(uid, tenant_id).await?;
                if credentials.is_empty() {
                    return Err(PasskeyError::NoCredentialsFound);
                }

                credentials
                    .into_iter()
                    .map(|c| self.credential_to_passkey(&c))
                    .collect::<Result<Vec<_>, _>>()?
            }
            None => vec![], // 可发现凭证认证
        };

        // 生成认证选项
        let (rcr, auth_state) = if allow_credentials.is_empty() {
            // 可发现凭证认证 (无用户名登录)
            self.webauthn
                .start_discoverable_authentication()
                .map_err(|e| PasskeyError::WebAuthnError(e.to_string()))?
        } else {
            self.webauthn
                .start_passkey_authentication(&allow_credentials)
                .map_err(|e| PasskeyError::WebAuthnError(e.to_string()))?
        };

        // 存储认证状态到 Redis
        let challenge_id = Uuid::new_v4().to_string();
        self.store_authentication_state(&challenge_id, user_id, tenant_id, &auth_state).await?;

        Ok(AuthenticationChallenge {
            challenge_id,
            options: rcr,
        })
    }

    /// 完成 Passkey 认证
    pub async fn finish_authentication(
        &self,
        challenge_id: &str,
        response: PublicKeyCredential,
    ) -> Result<AuthenticationResult, PasskeyError> {
        // 获取认证状态
        let (user_id, tenant_id, auth_state) = self
            .get_authentication_state(challenge_id)
            .await?
            .ok_or(PasskeyError::ChallengeExpired)?;

        // 获取凭证
        let credential_id = response.id.clone();
        let credential = self.get_credential_by_id(&credential_id, &tenant_id).await?
            .ok_or(PasskeyError::CredentialNotFound)?;

        let passkey = self.credential_to_passkey(&credential)?;

        // 验证认证响应
        let auth_result = self.webauthn
            .finish_passkey_authentication(&response, &auth_state)
            .map_err(|e| PasskeyError::WebAuthnError(e.to_string()))?;

        // 更新计数器
        self.update_credential_counter(
            &credential.id,
            auth_result.counter(),
        ).await?;

        // 删除认证状态
        self.delete_authentication_state(challenge_id).await?;

        Ok(AuthenticationResult {
            user_id: credential.user_id,
            tenant_id: credential.tenant_id,
            credential_id: credential.id,
        })
    }

    /// 列出用户的 Passkey
    pub async fn list_user_passkeys(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<Vec<PasskeyInfo>, PasskeyError> {
        let credentials = self.get_user_credentials(user_id, tenant_id).await?;

        Ok(credentials
            .into_iter()
            .map(|c| PasskeyInfo {
                id: c.id,
                device_name: c.device_name,
                created_at: c.created_at,
                last_used_at: c.last_used_at,
            })
            .collect())
    }

    /// 删除 Passkey
    pub async fn delete_passkey(
        &self,
        passkey_id: &str,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<(), PasskeyError> {
        sqlx::query(
            "DELETE FROM passkey_credentials
             WHERE id = $1 AND user_id = $2 AND tenant_id = $3"
        )
        .bind(passkey_id)
        .bind(user_id)
        .bind(tenant_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| PasskeyError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // 私有方法...

    async fn get_user_credentials(
        &self,
        user_id: &str,
        tenant_id: &str,
    ) -> Result<Vec<PasskeyCredential>, PasskeyError> {
        sqlx::query_as::<_, PasskeyCredential>(
            "SELECT * FROM passkey_credentials
             WHERE user_id = $1 AND tenant_id = $2"
        )
        .bind(user_id)
        .bind(tenant_id)
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| PasskeyError::DatabaseError(e.to_string()))
    }

    async fn save_credential(
        &self,
        user_id: &str,
        tenant_id: &str,
        passkey: &Passkey,
        device_name: Option<&str>,
    ) -> Result<PasskeyCredential, PasskeyError> {
        let id = Uuid::new_v4().to_string();
        let credential_id = passkey.cred_id().to_vec();
        let public_key = serde_json::to_vec(passkey)
            .map_err(|e| PasskeyError::SerializationError(e.to_string()))?;

        let credential = sqlx::query_as::<_, PasskeyCredential>(
            "INSERT INTO passkey_credentials
             (id, user_id, tenant_id, credential_id, public_key, counter, device_name, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
             RETURNING *"
        )
        .bind(&id)
        .bind(user_id)
        .bind(tenant_id)
        .bind(&credential_id)
        .bind(&public_key)
        .bind(0i32)
        .bind(device_name)
        .fetch_one(&self.db_pool)
        .await
        .map_err(|e| PasskeyError::DatabaseError(e.to_string()))?;

        Ok(credential)
    }

    fn credential_to_passkey(&self, credential: &PasskeyCredential) -> Result<Passkey, PasskeyError> {
        serde_json::from_slice(&credential.public_key)
            .map_err(|e| PasskeyError::SerializationError(e.to_string()))
    }

    async fn get_credential_by_id(
        &self,
        credential_id: &str,
        tenant_id: &str,
    ) -> Result<Option<PasskeyCredential>, PasskeyError> {
        let credential_bytes = URL_SAFE_NO_PAD.decode(credential_id)
            .map_err(|e| PasskeyError::InvalidCredentialId(e.to_string()))?;

        sqlx::query_as::<_, PasskeyCredential>(
            "SELECT * FROM passkey_credentials
             WHERE credential_id = $1 AND tenant_id = $2"
        )
        .bind(&credential_bytes)
        .bind(tenant_id)
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| PasskeyError::DatabaseError(e.to_string()))
    }

    async fn update_credential_counter(
        &self,
        credential_id: &str,
        counter: u32,
    ) -> Result<(), PasskeyError> {
        sqlx::query(
            "UPDATE passkey_credentials
             SET counter = $1, last_used_at = NOW()
             WHERE id = $2"
        )
        .bind(counter as i32)
        .bind(credential_id)
        .execute(&self.db_pool)
        .await
        .map_err(|e| PasskeyError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // Redis 状态管理方法省略...
    async fn store_registration_state(&self, _challenge_id: &str, _user_id: &str, _tenant_id: &str, _state: &PasskeyRegistration) -> Result<(), PasskeyError> {
        // 实现略
        Ok(())
    }

    async fn get_registration_state(&self, _challenge_id: &str) -> Result<Option<(String, String, PasskeyRegistration)>, PasskeyError> {
        // 实现略
        Ok(None)
    }

    async fn delete_registration_state(&self, _challenge_id: &str) -> Result<(), PasskeyError> {
        // 实现略
        Ok(())
    }

    async fn store_authentication_state(&self, _challenge_id: &str, _user_id: Option<&str>, _tenant_id: &str, _state: &PasskeyAuthentication) -> Result<(), PasskeyError> {
        // 实现略
        Ok(())
    }

    async fn get_authentication_state(&self, _challenge_id: &str) -> Result<Option<(Option<String>, String, PasskeyAuthentication)>, PasskeyError> {
        // 实现略
        Ok(None)
    }

    async fn delete_authentication_state(&self, _challenge_id: &str) -> Result<(), PasskeyError> {
        // 实现略
        Ok(())
    }
}

/// 认证结果
#[derive(Debug)]
pub struct AuthenticationResult {
    pub user_id: String,
    pub tenant_id: String,
    pub credential_id: String,
}

/// Passkey 信息
#[derive(Debug, Serialize, Deserialize)]
pub struct PasskeyInfo {
    pub id: String,
    pub device_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Passkey 错误
#[derive(Debug, thiserror::Error)]
pub enum PasskeyError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Invalid user ID: {0}")]
    InvalidUserId(String),
    #[error("WebAuthn error: {0}")]
    WebAuthnError(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Challenge expired or not found")]
    ChallengeExpired,
    #[error("No credentials found for user")]
    NoCredentialsFound,
    #[error("Credential not found")]
    CredentialNotFound,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Invalid credential ID: {0}")]
    InvalidCredentialId(String),
}
```

---

## 第三阶段：OAuth 社交登录

### 3.1 支持的提供商

| 提供商 | 协议 | 用途 |
|--------|------|------|
| Google | OAuth 2.0 + OIDC | 国际用户 |
| GitHub | OAuth 2.0 | 开发者 |
| 微信 | OAuth 2.0 | 中国用户 |
| 企业微信 | OAuth 2.0 | 企业用户 |
| 钉钉 | OAuth 2.0 | 企业用户 |

### 3.2 OAuth Manager 实现

```rust
// src/auth/oauth_manager.rs

use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OAuth 提供商枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    GitHub,
    Wechat,
    WechatWork,
    DingTalk,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::GitHub => "github",
            OAuthProvider::Wechat => "wechat",
            OAuthProvider::WechatWork => "wechat_work",
            OAuthProvider::DingTalk => "dingtalk",
        }
    }
}

/// OAuth 配置
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub userinfo_url: String,
    pub scopes: Vec<String>,
}

/// OAuth 用户信息
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthUserInfo {
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub raw_data: serde_json::Value,
}

/// OAuth 授权 URL 响应
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthorizationUrlResponse {
    pub url: String,
    pub state: String,
}

/// OAuth 管理器
pub struct OAuthManager {
    providers: HashMap<OAuthProvider, OAuthConfig>,
    clients: HashMap<OAuthProvider, BasicClient>,
    http_client: HttpClient,
    redis_pool: redis::Client,
    redirect_base_url: String,
}

impl OAuthManager {
    /// 创建新的 OAuth Manager
    pub fn new(
        configs: HashMap<OAuthProvider, OAuthConfig>,
        redis_pool: redis::Client,
        redirect_base_url: &str,
    ) -> Result<Self, OAuthError> {
        let mut clients = HashMap::new();

        for (provider, config) in &configs {
            let redirect_url = format!(
                "{}/api/v1/auth/oauth/{}/callback",
                redirect_base_url,
                provider.as_str()
            );

            let client = BasicClient::new(
                ClientId::new(config.client_id.clone()),
                Some(ClientSecret::new(config.client_secret.clone())),
                AuthUrl::new(config.auth_url.clone())
                    .map_err(|e| OAuthError::ConfigError(e.to_string()))?,
                Some(TokenUrl::new(config.token_url.clone())
                    .map_err(|e| OAuthError::ConfigError(e.to_string()))?),
            )
            .set_redirect_uri(
                RedirectUrl::new(redirect_url)
                    .map_err(|e| OAuthError::ConfigError(e.to_string()))?,
            );

            clients.insert(*provider, client);
        }

        Ok(Self {
            providers: configs,
            clients,
            http_client: HttpClient::new(),
            redis_pool,
            redirect_base_url: redirect_base_url.to_string(),
        })
    }

    /// 获取授权 URL
    pub async fn get_authorization_url(
        &self,
        provider: OAuthProvider,
        tenant_id: &str,
    ) -> Result<AuthorizationUrlResponse, OAuthError> {
        let client = self.clients.get(&provider)
            .ok_or(OAuthError::ProviderNotConfigured)?;

        let config = self.providers.get(&provider)
            .ok_or(OAuthError::ProviderNotConfigured)?;

        // 生成 PKCE
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // 构建授权请求
        let mut auth_request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // 添加 scopes
        for scope in &config.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.clone()));
        }

        let (auth_url, csrf_state) = auth_request.url();

        // 存储 state 和 PKCE verifier 到 Redis
        self.store_oauth_state(
            csrf_state.secret(),
            provider,
            tenant_id,
            pkce_verifier.secret(),
        ).await?;

        Ok(AuthorizationUrlResponse {
            url: auth_url.to_string(),
            state: csrf_state.secret().clone(),
        })
    }

    /// 处理回调
    pub async fn handle_callback(
        &self,
        provider: OAuthProvider,
        code: &str,
        state: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        // 获取并验证 state
        let (stored_provider, tenant_id, pkce_verifier) = self
            .get_oauth_state(state)
            .await?
            .ok_or(OAuthError::InvalidState)?;

        if stored_provider != provider {
            return Err(OAuthError::ProviderMismatch);
        }

        // 删除 state
        self.delete_oauth_state(state).await?;

        // 交换 code 获取 token
        let client = self.clients.get(&provider)
            .ok_or(OAuthError::ProviderNotConfigured)?;

        let token_result = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| OAuthError::TokenExchangeFailed(e.to_string()))?;

        let access_token = token_result.access_token().secret();

        // 获取用户信息
        let user_info = self.fetch_user_info(provider, access_token).await?;

        Ok(user_info)
    }

    /// 获取用户信息
    async fn fetch_user_info(
        &self,
        provider: OAuthProvider,
        access_token: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        let config = self.providers.get(&provider)
            .ok_or(OAuthError::ProviderNotConfigured)?;

        match provider {
            OAuthProvider::Google => self.fetch_google_user_info(access_token, &config.userinfo_url).await,
            OAuthProvider::GitHub => self.fetch_github_user_info(access_token, &config.userinfo_url).await,
            OAuthProvider::Wechat => self.fetch_wechat_user_info(access_token, &config.userinfo_url).await,
            OAuthProvider::WechatWork => self.fetch_wechat_work_user_info(access_token, &config.userinfo_url).await,
            OAuthProvider::DingTalk => self.fetch_dingtalk_user_info(access_token, &config.userinfo_url).await,
        }
    }

    /// 获取 Google 用户信息
    async fn fetch_google_user_info(
        &self,
        access_token: &str,
        userinfo_url: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        #[derive(Deserialize)]
        struct GoogleUser {
            sub: String,
            email: Option<String>,
            name: Option<String>,
            picture: Option<String>,
        }

        let response = self.http_client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OAuthError::UserInfoFetchFailed(e.to_string()))?;

        let user: GoogleUser = response.json().await
            .map_err(|e| OAuthError::UserInfoFetchFailed(e.to_string()))?;

        Ok(OAuthUserInfo {
            provider: OAuthProvider::Google,
            provider_user_id: user.sub,
            email: user.email,
            name: user.name,
            avatar_url: user.picture,
            raw_data: serde_json::json!({}),
        })
    }

    /// 获取 GitHub 用户信息
    async fn fetch_github_user_info(
        &self,
        access_token: &str,
        userinfo_url: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        #[derive(Deserialize)]
        struct GitHubUser {
            id: i64,
            email: Option<String>,
            name: Option<String>,
            avatar_url: Option<String>,
            login: String,
        }

        let response = self.http_client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .header("User-Agent", "CUBA-ERP")
            .send()
            .await
            .map_err(|e| OAuthError::UserInfoFetchFailed(e.to_string()))?;

        let user: GitHubUser = response.json().await
            .map_err(|e| OAuthError::UserInfoFetchFailed(e.to_string()))?;

        Ok(OAuthUserInfo {
            provider: OAuthProvider::GitHub,
            provider_user_id: user.id.to_string(),
            email: user.email,
            name: user.name.or(Some(user.login)),
            avatar_url: user.avatar_url,
            raw_data: serde_json::json!({}),
        })
    }

    /// 获取微信用户信息
    async fn fetch_wechat_user_info(
        &self,
        access_token: &str,
        userinfo_url: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        #[derive(Deserialize)]
        struct WechatUser {
            openid: String,
            unionid: Option<String>,
            nickname: Option<String>,
            headimgurl: Option<String>,
        }

        let url = format!("{}?access_token={}", userinfo_url, access_token);
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| OAuthError::UserInfoFetchFailed(e.to_string()))?;

        let user: WechatUser = response.json().await
            .map_err(|e| OAuthError::UserInfoFetchFailed(e.to_string()))?;

        Ok(OAuthUserInfo {
            provider: OAuthProvider::Wechat,
            provider_user_id: user.unionid.unwrap_or(user.openid),
            email: None,
            name: user.nickname,
            avatar_url: user.headimgurl,
            raw_data: serde_json::json!({}),
        })
    }

    /// 获取企业微信用户信息 (简化实现)
    async fn fetch_wechat_work_user_info(
        &self,
        _access_token: &str,
        _userinfo_url: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        // 企业微信需要更复杂的实现
        Err(OAuthError::NotImplemented)
    }

    /// 获取钉钉用户信息 (简化实现)
    async fn fetch_dingtalk_user_info(
        &self,
        _access_token: &str,
        _userinfo_url: &str,
    ) -> Result<OAuthUserInfo, OAuthError> {
        // 钉钉需要更复杂的实现
        Err(OAuthError::NotImplemented)
    }

    // Redis 状态管理
    async fn store_oauth_state(
        &self,
        state: &str,
        provider: OAuthProvider,
        tenant_id: &str,
        pkce_verifier: &str,
    ) -> Result<(), OAuthError> {
        use redis::AsyncCommands;

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| OAuthError::StorageError(e.to_string()))?;

        let key = format!("oauth:state:{}", state);
        let value = serde_json::json!({
            "provider": provider.as_str(),
            "tenant_id": tenant_id,
            "pkce_verifier": pkce_verifier,
        });

        conn.set_ex::<_, _, ()>(&key, value.to_string(), 600).await
            .map_err(|e| OAuthError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn get_oauth_state(
        &self,
        state: &str,
    ) -> Result<Option<(OAuthProvider, String, String)>, OAuthError> {
        use redis::AsyncCommands;

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| OAuthError::StorageError(e.to_string()))?;

        let key = format!("oauth:state:{}", state);
        let json: Option<String> = conn.get(&key).await
            .map_err(|e| OAuthError::StorageError(e.to_string()))?;

        match json {
            Some(j) => {
                let data: serde_json::Value = serde_json::from_str(&j)
                    .map_err(|e| OAuthError::StorageError(e.to_string()))?;

                let provider_str = data["provider"].as_str()
                    .ok_or(OAuthError::InvalidState)?;
                let provider = match provider_str {
                    "google" => OAuthProvider::Google,
                    "github" => OAuthProvider::GitHub,
                    "wechat" => OAuthProvider::Wechat,
                    "wechat_work" => OAuthProvider::WechatWork,
                    "dingtalk" => OAuthProvider::DingTalk,
                    _ => return Err(OAuthError::InvalidState),
                };

                let tenant_id = data["tenant_id"].as_str()
                    .ok_or(OAuthError::InvalidState)?
                    .to_string();
                let pkce_verifier = data["pkce_verifier"].as_str()
                    .ok_or(OAuthError::InvalidState)?
                    .to_string();

                Ok(Some((provider, tenant_id, pkce_verifier)))
            }
            None => Ok(None),
        }
    }

    async fn delete_oauth_state(&self, state: &str) -> Result<(), OAuthError> {
        use redis::AsyncCommands;

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| OAuthError::StorageError(e.to_string()))?;

        let key = format!("oauth:state:{}", state);
        conn.del::<_, ()>(&key).await
            .map_err(|e| OAuthError::StorageError(e.to_string()))?;

        Ok(())
    }
}

/// OAuth 错误
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Provider not configured")]
    ProviderNotConfigured,
    #[error("Invalid state")]
    InvalidState,
    #[error("Provider mismatch")]
    ProviderMismatch,
    #[error("Token exchange failed: {0}")]
    TokenExchangeFailed(String),
    #[error("User info fetch failed: {0}")]
    UserInfoFetchFailed(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Not implemented")]
    NotImplemented,
}
```

---

## 第四阶段：安全增强

### 4.1 安全功能

| 功能 | 描述 | 优先级 |
|------|------|--------|
| 设备指纹 | 识别用户设备 | 高 |
| 速率限制 | 防止暴力破解 | 高 |
| 异常检测 | 识别可疑登录 | 中 |
| IP 分析 | 地理位置和风险评估 | 中 |
| 行为分析 | 用户行为模式 | 低 |

### 4.2 Security Manager 实现

```rust
// src/auth/security_manager.rs

use chrono::{DateTime, Duration, Utc};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// 设备信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeviceInfo {
    pub fingerprint: String,
    pub user_agent: String,
    pub platform: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub screen_resolution: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
}

/// 登录上下文
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginContext {
    pub ip_address: IpAddr,
    pub device: DeviceInfo,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub tenant_id: String,
}

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 风险评估结果
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub level: RiskLevel,
    pub score: f64,
    pub factors: Vec<RiskFactor>,
    pub require_mfa: bool,
    pub require_captcha: bool,
    pub block: bool,
}

/// 风险因素
#[derive(Debug, Serialize, Deserialize)]
pub struct RiskFactor {
    pub name: String,
    pub weight: f64,
    pub description: String,
}

/// 速率限制结果
#[derive(Debug)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_at: DateTime<Utc>,
    pub retry_after: Option<u64>,
}

/// 安全管理器
pub struct SecurityManager {
    redis_pool: redis::Client,
    /// 登录尝试限制 (每 15 分钟)
    login_attempt_limit: u32,
    /// IP 限制 (每小时)
    ip_limit: u32,
    /// 设备限制 (每小时)
    device_limit: u32,
}

impl SecurityManager {
    pub fn new(
        redis_pool: redis::Client,
        login_attempt_limit: u32,
        ip_limit: u32,
        device_limit: u32,
    ) -> Self {
        Self {
            redis_pool,
            login_attempt_limit,
            ip_limit,
            device_limit,
        }
    }

    /// 检查速率限制
    pub async fn check_rate_limit(
        &self,
        context: &LoginContext,
    ) -> Result<RateLimitResult, SecurityError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        // 检查 IP 限制
        let ip_key = format!("ratelimit:ip:{}", context.ip_address);
        let ip_count: u32 = conn.get(&ip_key).await.unwrap_or(0);

        if ip_count >= self.ip_limit {
            let ttl: i64 = conn.ttl(&ip_key).await.unwrap_or(3600);
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at: Utc::now() + Duration::seconds(ttl),
                retry_after: Some(ttl as u64),
            });
        }

        // 检查设备限制
        let device_key = format!("ratelimit:device:{}", context.device.fingerprint);
        let device_count: u32 = conn.get(&device_key).await.unwrap_or(0);

        if device_count >= self.device_limit {
            let ttl: i64 = conn.ttl(&device_key).await.unwrap_or(3600);
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at: Utc::now() + Duration::seconds(ttl),
                retry_after: Some(ttl as u64),
            });
        }

        // 检查用户登录尝试限制
        if let Some(ref user_id) = context.user_id {
            let user_key = format!(
                "ratelimit:login:{}:{}",
                context.tenant_id, user_id
            );
            let user_count: u32 = conn.get(&user_key).await.unwrap_or(0);

            if user_count >= self.login_attempt_limit {
                let ttl: i64 = conn.ttl(&user_key).await.unwrap_or(900);
                return Ok(RateLimitResult {
                    allowed: false,
                    remaining: 0,
                    reset_at: Utc::now() + Duration::seconds(ttl),
                    retry_after: Some(ttl as u64),
                });
            }
        }

        Ok(RateLimitResult {
            allowed: true,
            remaining: self.login_attempt_limit.saturating_sub(
                context.user_id.as_ref().map(|_| 1).unwrap_or(0)
            ),
            reset_at: Utc::now() + Duration::minutes(15),
            retry_after: None,
        })
    }

    /// 记录登录尝试
    pub async fn record_login_attempt(
        &self,
        context: &LoginContext,
        success: bool,
    ) -> Result<(), SecurityError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        // 记录 IP
        let ip_key = format!("ratelimit:ip:{}", context.ip_address);
        conn.incr::<_, _, ()>(&ip_key, 1).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;
        conn.expire::<_, ()>(&ip_key, 3600).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        // 记录设备
        let device_key = format!("ratelimit:device:{}", context.device.fingerprint);
        conn.incr::<_, _, ()>(&device_key, 1).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;
        conn.expire::<_, ()>(&device_key, 3600).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        // 记录用户登录尝试 (只在失败时)
        if !success {
            if let Some(ref user_id) = context.user_id {
                let user_key = format!(
                    "ratelimit:login:{}:{}",
                    context.tenant_id, user_id
                );
                conn.incr::<_, _, ()>(&user_key, 1).await
                    .map_err(|e| SecurityError::StorageError(e.to_string()))?;
                conn.expire::<_, ()>(&user_key, 900).await
                    .map_err(|e| SecurityError::StorageError(e.to_string()))?;
            }
        } else {
            // 成功登录,清除失败计数
            if let Some(ref user_id) = context.user_id {
                let user_key = format!(
                    "ratelimit:login:{}:{}",
                    context.tenant_id, user_id
                );
                conn.del::<_, ()>(&user_key).await
                    .map_err(|e| SecurityError::StorageError(e.to_string()))?;
            }
        }

        // 存储登录历史
        self.store_login_history(context, success).await?;

        Ok(())
    }

    /// 评估风险
    pub async fn assess_risk(
        &self,
        context: &LoginContext,
    ) -> Result<RiskAssessment, SecurityError> {
        let mut factors = Vec::new();
        let mut total_score = 0.0;

        // 1. 检查是否是新设备
        let is_new_device = self.is_new_device(context).await?;
        if is_new_device {
            factors.push(RiskFactor {
                name: "new_device".to_string(),
                weight: 0.3,
                description: "从新设备登录".to_string(),
            });
            total_score += 0.3;
        }

        // 2. 检查是否是新 IP
        let is_new_ip = self.is_new_ip(context).await?;
        if is_new_ip {
            factors.push(RiskFactor {
                name: "new_ip".to_string(),
                weight: 0.2,
                description: "从新 IP 地址登录".to_string(),
            });
            total_score += 0.2;
        }

        // 3. 检查登录时间是否异常
        let is_unusual_time = self.is_unusual_time(context).await?;
        if is_unusual_time {
            factors.push(RiskFactor {
                name: "unusual_time".to_string(),
                weight: 0.15,
                description: "在不常见的时间登录".to_string(),
            });
            total_score += 0.15;
        }

        // 4. 检查是否有多次失败尝试
        let failed_attempts = self.get_failed_attempts(context).await?;
        if failed_attempts > 0 {
            let weight = (failed_attempts as f64 * 0.1).min(0.5);
            factors.push(RiskFactor {
                name: "failed_attempts".to_string(),
                weight,
                description: format!("有 {} 次失败的登录尝试", failed_attempts),
            });
            total_score += weight;
        }

        // 5. 检查 IP 信誉 (简化实现)
        let ip_reputation = self.check_ip_reputation(&context.ip_address).await?;
        if ip_reputation < 0.5 {
            factors.push(RiskFactor {
                name: "suspicious_ip".to_string(),
                weight: 0.4,
                description: "IP 地址信誉较低".to_string(),
            });
            total_score += 0.4;
        }

        // 确定风险等级
        let level = match total_score {
            s if s < 0.2 => RiskLevel::Low,
            s if s < 0.5 => RiskLevel::Medium,
            s if s < 0.8 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Ok(RiskAssessment {
            level,
            score: total_score,
            factors,
            require_mfa: total_score >= 0.5,
            require_captcha: total_score >= 0.3,
            block: total_score >= 0.9,
        })
    }

    /// 检查是否是新设备
    async fn is_new_device(&self, context: &LoginContext) -> Result<bool, SecurityError> {
        if context.user_id.is_none() {
            return Ok(true);
        }

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        let key = format!(
            "user:{}:{}:devices",
            context.tenant_id,
            context.user_id.as_ref().unwrap()
        );

        let is_member: bool = conn.sismember(&key, &context.device.fingerprint).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        Ok(!is_member)
    }

    /// 检查是否是新 IP
    async fn is_new_ip(&self, context: &LoginContext) -> Result<bool, SecurityError> {
        if context.user_id.is_none() {
            return Ok(true);
        }

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        let key = format!(
            "user:{}:{}:ips",
            context.tenant_id,
            context.user_id.as_ref().unwrap()
        );

        let is_member: bool = conn.sismember(&key, context.ip_address.to_string()).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        Ok(!is_member)
    }

    /// 检查是否是异常时间
    async fn is_unusual_time(&self, context: &LoginContext) -> Result<bool, SecurityError> {
        let hour = context.timestamp.hour();
        // 简化: 凌晨 2-5 点视为异常时间
        Ok((2..=5).contains(&hour))
    }

    /// 获取失败尝试次数
    async fn get_failed_attempts(&self, context: &LoginContext) -> Result<u32, SecurityError> {
        if context.user_id.is_none() {
            return Ok(0);
        }

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        let key = format!(
            "ratelimit:login:{}:{}",
            context.tenant_id,
            context.user_id.as_ref().unwrap()
        );

        let count: u32 = conn.get(&key).await.unwrap_or(0);
        Ok(count)
    }

    /// 检查 IP 信誉 (简化实现)
    async fn check_ip_reputation(&self, _ip: &IpAddr) -> Result<f64, SecurityError> {
        // 实际应该调用 IP 信誉服务
        // 这里返回默认值
        Ok(1.0)
    }

    /// 存储登录历史
    async fn store_login_history(
        &self,
        context: &LoginContext,
        success: bool,
    ) -> Result<(), SecurityError> {
        if context.user_id.is_none() {
            return Ok(());
        }

        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        let user_id = context.user_id.as_ref().unwrap();

        // 记录设备
        let devices_key = format!("user:{}:{}:devices", context.tenant_id, user_id);
        conn.sadd::<_, _, ()>(&devices_key, &context.device.fingerprint).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        // 记录 IP
        let ips_key = format!("user:{}:{}:ips", context.tenant_id, user_id);
        conn.sadd::<_, _, ()>(&ips_key, context.ip_address.to_string()).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        // 记录登录历史
        let history_key = format!("user:{}:{}:login_history", context.tenant_id, user_id);
        let history_entry = serde_json::json!({
            "timestamp": context.timestamp.to_rfc3339(),
            "ip": context.ip_address.to_string(),
            "device": context.device.fingerprint,
            "success": success,
        });

        conn.lpush::<_, _, ()>(&history_key, history_entry.to_string()).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;
        conn.ltrim::<_, ()>(&history_key, 0, 99).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        Ok(())
    }

    /// 获取用户登录历史
    pub async fn get_login_history(
        &self,
        user_id: &str,
        tenant_id: &str,
        limit: usize,
    ) -> Result<Vec<LoginHistoryEntry>, SecurityError> {
        let mut conn = self.redis_pool.get_async_connection().await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        let key = format!("user:{}:{}:login_history", tenant_id, user_id);
        let entries: Vec<String> = conn.lrange(&key, 0, limit as isize - 1).await
            .map_err(|e| SecurityError::StorageError(e.to_string()))?;

        let history: Vec<LoginHistoryEntry> = entries
            .iter()
            .filter_map(|e| serde_json::from_str(e).ok())
            .collect();

        Ok(history)
    }
}

/// 登录历史条目
#[derive(Debug, Serialize, Deserialize)]
pub struct LoginHistoryEntry {
    pub timestamp: String,
    pub ip: String,
    pub device: String,
    pub success: bool,
}

/// 安全错误
#[derive(Debug, thiserror::Error)]
pub enum SecurityError {
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}
```

---

## 数据库设计

### 用户表 (已存在,需扩展)

```sql
-- 添加 OAuth 关联
ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_provider VARCHAR(50);
ALTER TABLE users ADD COLUMN IF NOT EXISTS oauth_provider_user_id VARCHAR(255);
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url TEXT;

-- 创建 OAuth 关联索引
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_oauth
ON users(tenant_id, oauth_provider, oauth_provider_user_id)
WHERE oauth_provider IS NOT NULL;
```

### Passkey 凭证表

```sql
CREATE TABLE IF NOT EXISTS passkey_credentials (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    tenant_id VARCHAR(36) NOT NULL,
    credential_id BYTEA NOT NULL,
    public_key BYTEA NOT NULL,
    counter INTEGER NOT NULL DEFAULT 0,
    aaguid BYTEA,
    device_name VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,

    CONSTRAINT fk_passkey_user
        FOREIGN KEY (user_id, tenant_id)
        REFERENCES users(id, tenant_id)
        ON DELETE CASCADE
);

CREATE INDEX idx_passkey_user ON passkey_credentials(user_id, tenant_id);
CREATE UNIQUE INDEX idx_passkey_credential_id ON passkey_credentials(credential_id, tenant_id);
```

### OAuth 关联表

```sql
CREATE TABLE IF NOT EXISTS oauth_connections (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    tenant_id VARCHAR(36) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    provider_email VARCHAR(255),
    provider_name VARCHAR(255),
    provider_avatar_url TEXT,
    access_token TEXT,
    refresh_token TEXT,
    token_expires_at TIMESTAMP WITH TIME ZONE,
    raw_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_oauth_user
        FOREIGN KEY (user_id, tenant_id)
        REFERENCES users(id, tenant_id)
        ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_oauth_provider
ON oauth_connections(tenant_id, provider, provider_user_id);
CREATE INDEX idx_oauth_user ON oauth_connections(user_id, tenant_id);
```

### 设备表

```sql
CREATE TABLE IF NOT EXISTS user_devices (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL,
    tenant_id VARCHAR(36) NOT NULL,
    device_fingerprint VARCHAR(255) NOT NULL,
    device_name VARCHAR(255),
    user_agent TEXT,
    platform VARCHAR(50),
    browser VARCHAR(100),
    os VARCHAR(100),
    is_trusted BOOLEAN NOT NULL DEFAULT FALSE,
    last_seen_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT fk_device_user
        FOREIGN KEY (user_id, tenant_id)
        REFERENCES users(id, tenant_id)
        ON DELETE CASCADE
);

CREATE UNIQUE INDEX idx_device_fingerprint
ON user_devices(user_id, tenant_id, device_fingerprint);
```

### 审计日志表

```sql
CREATE TABLE IF NOT EXISTS auth_audit_logs (
    id VARCHAR(36) PRIMARY KEY,
    tenant_id VARCHAR(36) NOT NULL,
    user_id VARCHAR(36),
    action VARCHAR(50) NOT NULL,
    ip_address INET NOT NULL,
    device_fingerprint VARCHAR(255),
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    error_code VARCHAR(50),
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- 按时间分区 (推荐)
CREATE INDEX idx_audit_tenant_time ON auth_audit_logs(tenant_id, created_at DESC);
CREATE INDEX idx_audit_user_time ON auth_audit_logs(user_id, created_at DESC) WHERE user_id IS NOT NULL;
CREATE INDEX idx_audit_action ON auth_audit_logs(action, created_at DESC);
```

---

## API 设计

### 认证 API

```yaml
# 第一阶段 - JWT 增强
POST   /api/v1/auth/login              # 用户登录
POST   /api/v1/auth/logout             # 用户登出
POST   /api/v1/auth/refresh            # 刷新 Token
POST   /api/v1/auth/revoke             # 撤销 Token
POST   /api/v1/auth/revoke-all         # 撤销所有 Token
GET    /api/v1/auth/sessions           # 获取会话列表
DELETE /api/v1/auth/sessions/{id}      # 终止指定会话

# 第二阶段 - Passkey
POST   /api/v1/auth/passkey/register/start     # 开始注册 Passkey
POST   /api/v1/auth/passkey/register/finish    # 完成注册 Passkey
POST   /api/v1/auth/passkey/authenticate/start # 开始 Passkey 认证
POST   /api/v1/auth/passkey/authenticate/finish# 完成 Passkey 认证
GET    /api/v1/auth/passkey/list               # 列出用户 Passkey
DELETE /api/v1/auth/passkey/{id}               # 删除 Passkey

# 第三阶段 - OAuth
GET    /api/v1/auth/oauth/{provider}           # 获取授权 URL
GET    /api/v1/auth/oauth/{provider}/callback  # OAuth 回调
POST   /api/v1/auth/oauth/link                 # 关联 OAuth 账号
DELETE /api/v1/auth/oauth/{provider}           # 解除关联
GET    /api/v1/auth/oauth/connections          # 获取关联列表

# 第四阶段 - 安全
GET    /api/v1/auth/devices                    # 获取设备列表
POST   /api/v1/auth/devices/{id}/trust         # 信任设备
DELETE /api/v1/auth/devices/{id}               # 删除设备
GET    /api/v1/auth/login-history              # 获取登录历史
POST   /api/v1/auth/mfa/totp/setup             # 设置 TOTP
POST   /api/v1/auth/mfa/totp/verify            # 验证 TOTP
DELETE /api/v1/auth/mfa/totp                   # 删除 TOTP
```

---

## 技术栈

### 后端

| 组件 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 1.75+ |
| Web 框架 | Tonic (gRPC) | 0.10+ |
| HTTP 框架 | Axum | 0.7+ |
| 数据库 | PostgreSQL | 15+ |
| 缓存 | Redis | 7+ |
| WebAuthn | webauthn-rs | 0.5+ |
| JWT | jsonwebtoken | 9+ |
| OAuth | oauth2 | 4+ |

### 依赖配置

```toml
# Cargo.toml 新增依赖
[dependencies]
# WebAuthn
webauthn-rs = { version = "0.5", features = ["danger-allow-state-serialisation"] }

# OAuth
oauth2 = "4"

# 安全
argon2 = "0.5"
rand = "0.8"
sha2 = "0.10"

# 序列化
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# 时间
chrono = { version = "0.4", features = ["serde"] }

# UUID
uuid = { version = "1", features = ["v4", "serde"] }

# 错误处理
thiserror = "1"
anyhow = "1"

# 异步
tokio = { version = "1", features = ["full"] }

# 数据库
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "chrono", "uuid"] }

# Redis
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }

# HTTP 客户端
reqwest = { version = "0.11", features = ["json"] }

# Base64
base64 = "0.21"
```

---

## 迁移计划

### 阶段一：JWT 增强 (2-3 周)

```
Week 1:
├── 修改 Token 结构,添加 JTI
├── 实现 TokenManager
├── 添加 Redis Token 黑名单
└── 更新登录/登出 API

Week 2:
├── 实现会话管理
├── 添加 Token 撤销 API
├── 更新 Envoy 认证过滤器
└── 测试和文档

Week 3:
├── 性能测试
├── 安全审计
└── 部署到测试环境
```

### 阶段二：Passkey (3-4 周)

```
Week 1:
├── 添加 webauthn-rs 依赖
├── 创建 PasskeyManager
├── 设计数据库表
└── 实现注册流程

Week 2:
├── 实现认证流程
├── 添加 Passkey 管理 API
├── 前端集成 (需要前端配合)
└── 单元测试

Week 3-4:
├── 跨浏览器测试
├── 安全审计
├── 文档完善
└── 部署
```

### 阶段三：OAuth (2-3 周)

```
Week 1:
├── 实现 OAuthManager
├── 添加 Google 登录
├── 添加 GitHub 登录
└── 创建账号关联表

Week 2:
├── 添加微信登录 (可选)
├── 账号合并逻辑
├── 前端集成
└── 测试

Week 3:
├── 安全审计
├── 错误处理完善
└── 部署
```

### 阶段四：安全增强 (2-3 周)

```
Week 1:
├── 实现 SecurityManager
├── 添加设备指纹
├── 实现速率限制
└── 基础风险评估

Week 2:
├── 添加登录历史
├── 设备管理 API
├── MFA 准备工作
└── 测试

Week 3:
├── 监控和告警
├── 性能优化
└── 文档和部署
```

---

## 配置示例

### 环境变量

```env
# JWT 配置
JWT_SECRET=your-super-secret-key-at-least-32-chars
JWT_ACCESS_TOKEN_TTL_MINUTES=15
JWT_REFRESH_TOKEN_TTL_DAYS=7

# Redis 配置
REDIS_URL=redis://localhost:6379

# WebAuthn 配置
WEBAUTHN_RP_ID=your-domain.com
WEBAUTHN_RP_ORIGIN=https://your-domain.com
WEBAUTHN_RP_NAME=CUBA ERP

# OAuth - Google
OAUTH_GOOGLE_CLIENT_ID=your-google-client-id
OAUTH_GOOGLE_CLIENT_SECRET=your-google-client-secret

# OAuth - GitHub
OAUTH_GITHUB_CLIENT_ID=your-github-client-id
OAUTH_GITHUB_CLIENT_SECRET=your-github-client-secret

# OAuth - 微信 (可选)
OAUTH_WECHAT_APP_ID=your-wechat-app-id
OAUTH_WECHAT_APP_SECRET=your-wechat-app-secret

# 安全配置
SECURITY_LOGIN_ATTEMPT_LIMIT=5
SECURITY_IP_LIMIT=100
SECURITY_DEVICE_LIMIT=50
```

### 配置文件

```yaml
# config/auth.yaml
auth:
  jwt:
    secret: ${JWT_SECRET}
    access_token_ttl_minutes: 15
    refresh_token_ttl_days: 7
    issuer: cuba-auth
    audience:
      - cuba-api

  webauthn:
    rp_id: ${WEBAUTHN_RP_ID}
    rp_origin: ${WEBAUTHN_RP_ORIGIN}
    rp_name: ${WEBAUTHN_RP_NAME}

  oauth:
    redirect_base_url: ${OAUTH_REDIRECT_BASE_URL}
    providers:
      google:
        enabled: true
        client_id: ${OAUTH_GOOGLE_CLIENT_ID}
        client_secret: ${OAUTH_GOOGLE_CLIENT_SECRET}
        auth_url: https://accounts.google.com/o/oauth2/v2/auth
        token_url: https://oauth2.googleapis.com/token
        userinfo_url: https://openidconnect.googleapis.com/v1/userinfo
        scopes:
          - openid
          - email
          - profile

      github:
        enabled: true
        client_id: ${OAUTH_GITHUB_CLIENT_ID}
        client_secret: ${OAUTH_GITHUB_CLIENT_SECRET}
        auth_url: https://github.com/login/oauth/authorize
        token_url: https://github.com/login/oauth/access_token
        userinfo_url: https://api.github.com/user
        scopes:
          - read:user
          - user:email

  security:
    rate_limit:
      login_attempt_limit: 5
      ip_limit: 100
      device_limit: 50
    risk_assessment:
      enabled: true
      mfa_threshold: 0.5
      captcha_threshold: 0.3
      block_threshold: 0.9
```

---

## 总结

### 实施优先级

| 阶段 | 功能 | 优先级 | 工作量 |
|------|------|--------|--------|
| 1 | JWT + JTI + 撤销 | 高 | 2-3 周 |
| 2 | Passkey/WebAuthn | 中 | 3-4 周 |
| 3 | OAuth 社交登录 | 中 | 2-3 周 |
| 4 | 安全增强 | 低 | 2-3 周 |

### 风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 向后兼容 | 高 | 渐进式迁移,保留旧 API |
| 性能影响 | 中 | Redis 缓存,异步处理 |
| 安全漏洞 | 高 | 代码审查,渗透测试 |
| 用户体验 | 中 | 充分测试,灰度发布 |

### 预期收益

1. **安全性提升**: Token 可撤销,减少安全风险
2. **用户体验**: Passkey 无密码登录更便捷
3. **用户增长**: 社交登录降低注册门槛
4. **合规性**: 满足企业安全要求

---

**文档版本**: 1.0.0
**创建日期**: 2026-01-20
**状态**: 待实施
**作者**: Claude AI Assistant
