//! Auth Service gRPC Implementation
//!
//! 实现 proto 定义的 AuthService。

use crate::domain::DomainError;
use crate::application::commands::{
    AddPermissionCommand, AddPermissionHandler, AssignRoleCommand, AssignRoleHandler,
    ChangePasswordCommand, ChangePasswordHandler, CreateRoleCommand, CreateRoleHandler,
    DeleteRoleCommand, DeleteRoleHandler, ListPermissionsCommand, ListPermissionsHandler, ListRolesCommand, ListRolesHandler,
    ListUserSessionsCommand, ListUserSessionsHandler, RevokeSessionCommand, RevokeSessionHandler,
    CreateClientCommand, CreateClientHandler, ListClientsCommand, ListClientsHandler,
    LoginCommand, LoginHandler, RegisterCommand, RegisterHandler,
    ListUsersCommand, ListUsersHandler, ListUsersResponse as AppListUsersResponse,
    BulkCreateUsersCommand, BulkCreateUsersHandler, BulkCreateUsersResponse as AppBulkCreateUsersResponse,
    RemovePermissionFromRoleCommand, RemovePermissionFromRoleHandler,
    RemoveRoleFromUserCommand, RemoveRoleFromUserHandler,
    UpdateUserProfileCommand, UpdateUserProfileHandler, UpdateUserStatusCommand, UpdateUserStatusHandler,
    SendPasswordResetTokenCommand, SendPasswordResetTokenHandler, ResetPasswordCommand, ResetPasswordHandler,
    VerifyEmailCommand, VerifyEmailHandler, CreateAPIKeyCommand, CreateAPIKeyHandler, ListAPIKeysCommand, ListAPIKeysHandler,
    RevokeAPIKeyCommand, RevokeAPIKeyHandler, GetAuditLogsCommand, GetAuditLogsHandler,
    AuthorizeCommand, AuthorizeHandler, OAuth2TokenCommand, OAuth2TokenHandler,
    UserInfoCommand, UserInfoHandler,
    SocialLoginCommand, SocialLoginHandler,
    SSOLoginCommand, SSOLoginHandler,
    CreatePolicyCommand, CreatePolicyHandler, StatementDto,
    AttachPolicyToRoleCommand, AttachPolicyToRoleHandler,
    AttachPolicyToUserCommand, AttachPolicyToUserHandler,
    Enable2FACommand, Enable2FAHandler, Verify2FASetupCommand, Verify2FASetupHandler, Verify2FACodeCommand, Verify2FACodeHandler,
};
use crate::domain::repositories::{RoleRepository, UserRepository, RefreshTokenRepository, VerificationRepository, ApiKeyRepository, AuditLogRepository, SessionRepository, ClientRepository, PolicyRepository, RepositoryError};
use crate::domain::services::TokenService;
use crate::infrastructure::services::social_auth::SocialAuthService;
use crate::infrastructure::services::sso_service::SSOService;
use crate::proto::auth_service_server::AuthService;
use crate::proto::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};

/// AuthService gRPC 实现
pub struct AuthServiceImpl {
    register_handler: RegisterHandler,
    login_handler: LoginHandler,
    create_role_handler: CreateRoleHandler,
    assign_role_handler: AssignRoleHandler,
    add_permission_handler: AddPermissionHandler,
    change_password_handler: ChangePasswordHandler,
    update_user_profile_handler: UpdateUserProfileHandler,
    update_user_status_handler: UpdateUserStatusHandler,
    list_roles_handler: ListRolesHandler,
    list_permissions_handler: ListPermissionsHandler,
    delete_role_handler: DeleteRoleHandler,
    remove_role_from_user_handler: RemoveRoleFromUserHandler,
    remove_permission_from_role_handler: RemovePermissionFromRoleHandler,
    list_users_handler: ListUsersHandler,
    bulk_create_users_handler: BulkCreateUsersHandler,
    send_password_reset_token_handler: SendPasswordResetTokenHandler,
    reset_password_handler: ResetPasswordHandler,
    verify_email_handler: VerifyEmailHandler,
    create_api_key_handler: CreateAPIKeyHandler,
    list_api_keys_handler: ListAPIKeysHandler,
    revoke_api_key_handler: RevokeAPIKeyHandler,
    get_audit_logs_handler: GetAuditLogsHandler,
    authorize_handler: AuthorizeHandler,
    oauth2_token_handler: OAuth2TokenHandler,
    enable_2fa_handler: Enable2FAHandler,
    verify_2fa_setup_handler: Verify2FASetupHandler,
    verify_2fa_code_handler: Verify2FACodeHandler,
    token_service: Arc<dyn TokenService>,
    user_repo: Arc<dyn UserRepository>,
    role_repo: Arc<dyn RoleRepository>,
    verification_repo: Arc<dyn VerificationRepository>,
    api_key_repo: Arc<dyn ApiKeyRepository>,
    audit_repo: Arc<dyn AuditLogRepository>,
    session_repo: Arc<dyn SessionRepository>,
    client_repo: Arc<dyn ClientRepository>,
    
    // Session Handlers
    list_user_sessions_handler: ListUserSessionsHandler,
    revoke_session_handler: RevokeSessionHandler,
    
    // Client Handlers
    create_client_handler: CreateClientHandler,
    list_clients_handler: ListClientsHandler,

    // Policy Handlers
    create_policy_handler: CreatePolicyHandler,
    attach_policy_to_role_handler: AttachPolicyToRoleHandler,
    attach_policy_to_user_handler: AttachPolicyToUserHandler,
    
    policy_repo: Arc<dyn PolicyRepository>,

    // OIDC Handlers
    user_info_handler: UserInfoHandler,
    social_login_handler: SocialLoginHandler,
    sso_login_handler: SSOLoginHandler,
}

impl AuthServiceImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        role_repo: Arc<dyn RoleRepository>,
        verification_repo: Arc<dyn VerificationRepository>,
        api_key_repo: Arc<dyn ApiKeyRepository>,
        audit_repo: Arc<dyn AuditLogRepository>,
        session_repo: Arc<dyn SessionRepository>,
        client_repo: Arc<dyn ClientRepository>,
        policy_repo: Arc<dyn PolicyRepository>,
        token_service: Arc<dyn TokenService>,
        social_auth_service: Arc<SocialAuthService>,
        sso_service: Arc<SSOService>,
    ) -> Self {
        Self {
            register_handler: RegisterHandler::new(user_repo.clone()),
            login_handler: LoginHandler::new(user_repo.clone(), token_service.clone()),
            create_role_handler: CreateRoleHandler::new(role_repo.clone()),
            assign_role_handler: AssignRoleHandler::new(user_repo.clone(), role_repo.clone()),
            add_permission_handler: AddPermissionHandler::new(role_repo.clone()),
            change_password_handler: ChangePasswordHandler::new(user_repo.clone()),
            update_user_profile_handler: UpdateUserProfileHandler::new(user_repo.clone()),
            update_user_status_handler: UpdateUserStatusHandler::new(user_repo.clone()),
            list_roles_handler: ListRolesHandler::new(role_repo.clone()),
            list_permissions_handler: ListPermissionsHandler::new(role_repo.clone()),
            delete_role_handler: DeleteRoleHandler::new(role_repo.clone()),
            remove_role_from_user_handler: RemoveRoleFromUserHandler::new(user_repo.clone(), role_repo.clone()),
            remove_permission_from_role_handler: RemovePermissionFromRoleHandler::new(role_repo.clone()),
            list_users_handler: ListUsersHandler::new(user_repo.clone()),
            bulk_create_users_handler: BulkCreateUsersHandler::new(user_repo.clone()),
            send_password_reset_token_handler: SendPasswordResetTokenHandler::new(user_repo.clone(), verification_repo.clone()),
            reset_password_handler: ResetPasswordHandler::new(user_repo.clone(), verification_repo.clone()),
            verify_email_handler: VerifyEmailHandler::new(user_repo.clone(), verification_repo.clone()),
            create_api_key_handler: CreateAPIKeyHandler::new(api_key_repo.clone()),
            list_api_keys_handler: ListAPIKeysHandler::new(api_key_repo.clone()),
            revoke_api_key_handler: RevokeAPIKeyHandler::new(api_key_repo.clone()),
            get_audit_logs_handler: GetAuditLogsHandler::new(audit_repo.clone()),
            authorize_handler: AuthorizeHandler::new(verification_repo.clone()),
            oauth2_token_handler: OAuth2TokenHandler::new(verification_repo.clone(), user_repo.clone(), token_service.clone()),
            enable_2fa_handler: Enable2FAHandler::new(user_repo.clone()),
            verify_2fa_setup_handler: Verify2FASetupHandler::new(user_repo.clone()),
            verify_2fa_code_handler: Verify2FACodeHandler::new(user_repo.clone(), token_service.clone()),
            
            list_user_sessions_handler: ListUserSessionsHandler::new(session_repo.clone()),
            revoke_session_handler: RevokeSessionHandler::new(session_repo.clone()),
            create_client_handler: CreateClientHandler::new(client_repo.clone()),
            list_clients_handler: ListClientsHandler::new(client_repo.clone()),
            user_info_handler: UserInfoHandler::new(user_repo.clone(), token_service.clone()),
            social_login_handler: SocialLoginHandler::new(user_repo.clone(), token_service.clone(), social_auth_service),
            sso_login_handler: SSOLoginHandler::new(user_repo.clone(), token_service.clone(), sso_service),
            
            create_policy_handler: CreatePolicyHandler::new(policy_repo.clone()),
            attach_policy_to_role_handler: AttachPolicyToRoleHandler::new(policy_repo.clone()),
            attach_policy_to_user_handler: AttachPolicyToUserHandler::new(policy_repo.clone()),

            token_service,
            user_repo,
            role_repo,
            verification_repo,
            api_key_repo,
            audit_repo,
            session_repo,
            client_repo,
            policy_repo,
        }
    }

    fn map_user_info(&self, tenant_id: String, user: crate::application::dto::UserDto) -> UserInfo {
        UserInfo {
            tenant_id,
            user_id: user.user_id,
            username: user.username,
            email: user.email,
            display_name: user.display_name,
            avatar_url: user.avatar_url,
            email_verified: user.email_verified,
            is_active: user.is_active,
            roles: user.roles,
            created_at: Some(Self::to_timestamp(user.created_at)),
            updated_at: Some(Self::to_timestamp(user.updated_at)),
            last_login_at: user.last_login_at.map(Self::to_timestamp),
        }
    }

    fn to_timestamp(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
        prost_types::Timestamp {
            seconds: dt.timestamp(),
            nanos: dt.timestamp_subsec_nanos() as i32,
        }
    }
    // Helper to extract user_id from metadata or Authorization header
    fn get_user_id_from_metadata(&self, metadata: &tonic::metadata::MetadataMap) -> Result<String, Status> {
        // 1. Try x-user-id (internal/gateway usage)
        if let Some(user_id) = metadata.get("x-user-id") {
            if let Ok(id_str) = user_id.to_str() {
                return Ok(id_str.to_string());
            }
        }

        // 2. Try Authorization Bearer token
        if let Some(auth_val) = metadata.get("authorization") {
            if let Ok(auth_str) = auth_val.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    match self.token_service.validate_token(token) {
                        Ok(claims) => return Ok(claims.sub),
                        Err(e) => return Err(Status::unauthenticated(format!("Invalid token: {}", e))),
                    }
                }
            }
        }

        Err(Status::unauthenticated("User ID not found in metadata and missing/invalid Authorization token"))
    }

    fn map_error(e: DomainError) -> Status {
        match e {
            DomainError::InvalidCredentials | DomainError::AuthenticationFailed(_) | DomainError::TokenInvalid | DomainError::TokenExpired | DomainError::TokenRevoked | DomainError::RefreshTokenRevoked => {
                Status::unauthenticated(e.to_string())
            }
            DomainError::UserNotFound | DomainError::RoleNotFound(_) | DomainError::PermissionNotFound(_) | DomainError::NotFound(_) => {
                Status::not_found(e.to_string())
            }
            DomainError::UsernameAlreadyExists(_) | DomainError::EmailAlreadyExists(_) | DomainError::AlreadyExists(_) => {
                Status::already_exists(e.to_string())
            }
            DomainError::NotPermitted => {
                Status::permission_denied(e.to_string())
            }
            DomainError::InvalidInput(_) | DomainError::UsernameRequired | DomainError::InvalidEmailFormat | DomainError::PasswordTooShort | DomainError::PasswordNoUppercase | DomainError::PasswordNoDigit | DomainError::RoleNameRequired => {
                Status::invalid_argument(e.to_string())
            }
            DomainError::InfrastructureError(_) | DomainError::InternalError(_) => {
                Status::internal(e.to_string())
            }
            _ => Status::internal(e.to_string()),
        }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    /// 用户注册
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        let command = RegisterCommand {
            username: req.username,
            email: req.email,
            password: req.password,
        };

        let user_dto = self
            .register_handler
            .handle(command)
            .await
            .map_err(|e| Self::map_error(e.into()))?;

        let response = RegisterResponse {
            user: Some(self.map_user_info(req.tenant_id, user_dto)),
        };

        Ok(Response::new(response))
    }

    /// 用户登录
    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let command = LoginCommand {
            username: req.username,
            password: req.password,
            tenant_id: req.tenant_id.clone(),
        };

        let login_response = self
            .login_handler
            .handle(command)
            .await
            .map_err(|e| Self::map_error(e.into()))?;

        let response = LoginResponse {
            access_token: login_response.access_token.unwrap_or_default(),
            refresh_token: login_response.refresh_token.unwrap_or_default(),
            expires_in: login_response.expires_in.unwrap_or_default(),
            user: Some(self.map_user_info(req.tenant_id, login_response.user)),
            requires_2fa: login_response.requires_2fa,
            account_locked: false,
            lock_until: None,
            temp_token: login_response.temp_token.unwrap_or_default(),
        };

        Ok(Response::new(response))
    }

    /// 用户登出
    async fn logout(
        &self,
        request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        let req = request.into_inner();
        
        if !req.refresh_token.is_empty() {
            self.token_service.revoke_token(&req.refresh_token).await
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        Ok(Response::new(LogoutResponse { success: true }))
    }

    /// 刷新令牌
    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();

        let (access_token, refresh_token, expires_in) = self
            .token_service
            .refresh_tokens(&req.refresh_token)
            .await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        Ok(Response::new(RefreshTokenResponse {
            access_token,
            refresh_token,
            expires_in,
        }))
    }

    /// 验证令牌
    async fn validate_token(
        &self,
        request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        let req = request.into_inner();

        match self.token_service.validate_token(&req.access_token) {
            Ok(claims) => Ok(Response::new(ValidateTokenResponse {
                valid: true,
                user_id: claims.sub,
                tenant_id: claims.tid,
                roles: claims.roles,
                permissions: claims.permissions,
            })),
            Err(_) => Ok(Response::new(ValidateTokenResponse {
                valid: false,
                user_id: String::new(),
                tenant_id: String::new(),
                roles: vec![],
                permissions: vec![],
            })),
        }
    }

    /// 获取用户信息
    async fn get_user_info(
        &self,
        request: Request<GetUserInfoRequest>,
    ) -> Result<Response<GetUserInfoResponse>, Status> {
        let user_id_str = if request.get_ref().user_id.is_empty() {
            self.get_user_id_from_metadata(request.metadata())?
        } else {
            request.into_inner().user_id
        };

        let user_id: uuid::Uuid = user_id_str
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let user = self
            .user_repo
            .find_by_id(&crate::domain::value_objects::UserId::from(user_id))
            .await
            .map_err(|e| Self::map_error(e.into()))?
            .ok_or_else(|| Status::not_found("User not found"))?;

        Ok(Response::new(GetUserInfoResponse {
            user: Some(self.map_user_info(String::new(), crate::application::dto::UserDto::from_user(&user))),
        }))
    }

    /// 创建角色
    async fn create_role(
        &self,
        request: Request<CreateRoleRequest>,
    ) -> Result<Response<CreateRoleResponse>, Status> {
        let req = request.into_inner();

        let command = CreateRoleCommand {
            name: req.name,
            description: if req.description.is_empty() {
                None
            } else {
                Some(req.description)
            },
        };

        let role_dto = self
            .create_role_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateRoleResponse {
            role: Some(crate::proto::Role {
                role_id: role_dto.role_id,
                name: role_dto.name,
                description: role_dto.description.unwrap_or_default(),
                created_at: Some(Self::to_timestamp(role_dto.created_at)),
            }),
        }))
    }

    /// 分配角色给用户
    async fn assign_role_to_user(
        &self,
        request: Request<AssignRoleToUserRequest>,
    ) -> Result<Response<AssignRoleToUserResponse>, Status> {
        let req = request.into_inner();

        let command = AssignRoleCommand {
            user_id: req.user_id,
            role_id: req.role_id,
        };

        self.assign_role_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AssignRoleToUserResponse { success: true }))
    }

    /// 添加权限到角色
    async fn add_permission_to_role(
        &self,
        request: Request<AddPermissionToRoleRequest>,
    ) -> Result<Response<AddPermissionToRoleResponse>, Status> {
        let req = request.into_inner();

        let command = AddPermissionCommand {
            role_id: req.role_id,
            permission_id: req.permission_id,
        };

        self.add_permission_handler
            .handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AddPermissionToRoleResponse { success: true }))
    }

    /// 获取用户权限列表
    async fn get_user_permissions(
        &self,
        request: Request<GetUserPermissionsRequest>,
    ) -> Result<Response<GetUserPermissionsResponse>, Status> {
        let req = request.into_inner();

        let user_id: uuid::Uuid = req
            .user_id
            .parse()
            .map_err(|_| Status::invalid_argument("Invalid user_id"))?;

        let permissions = self
            .user_repo
            .get_user_permissions(&crate::domain::value_objects::UserId::from(user_id))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_permissions = permissions
            .into_iter()
            .map(|p| Permission {
                permission_id: String::new(),
                resource: p.resource().to_string(),
                actions: vec![p.action().to_string()],
                scope: String::new(),
                attributes: std::collections::HashMap::new(),
                description: String::new(),
            })
            .collect();

        Ok(Response::new(GetUserPermissionsResponse {
            permissions: proto_permissions,
        }))
    }

    // --- 新增 RPC Stub 实现 ---

    async fn change_password(
        &self,
        request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        let user_id = self.get_user_id_from_metadata(request.metadata())?;
    
        let req = request.into_inner();

        let command = ChangePasswordCommand {
            user_id: user_id.to_string(),
            current_password: req.old_password,
            new_password: req.new_password,
        };

        self.change_password_handler
            .handle(command)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(ChangePasswordResponse { 
            success: true,
            message: "Password changed successfully".to_string() 
        }))
    }

    async fn reset_password(
        &self,
        request: Request<ResetPasswordRequest>,
    ) -> Result<Response<ResetPasswordResponse>, Status> {
        let req = request.into_inner();
        
        self.reset_password_handler.handle(ResetPasswordCommand {
            token: req.token,
            new_password: req.new_password,
        })
        .await
        .map_err(Self::map_error)?;

        Ok(Response::new(ResetPasswordResponse { success: true, message: "Password reset successful".to_string() }))
    }

    async fn send_password_reset_token(
        &self,
        request: Request<SendPasswordResetTokenRequest>,
    ) -> Result<Response<SendPasswordResetTokenResponse>, Status> {
        let req = request.into_inner();
        
        self.send_password_reset_token_handler.handle(SendPasswordResetTokenCommand {
            email: req.email,
        })
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SendPasswordResetTokenResponse { success: true, message: "Reset token sent".to_string() }))
    }

    async fn verify_email(
        &self,
        request: Request<VerifyEmailRequest>,
    ) -> Result<Response<VerifyEmailResponse>, Status> {
        let req = request.into_inner();
        
        self.verify_email_handler.handle(VerifyEmailCommand {
            token: req.token,
        })
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(VerifyEmailResponse { success: true, message: "Email verified successful".to_string() }))
    }

    async fn update_user_profile(
        &self,
        request: Request<UpdateUserProfileRequest>,
    ) -> Result<Response<UpdateUserProfileResponse>, Status> {
        let user_id = if request.get_ref().user_id.is_empty() {
            self.get_user_id_from_metadata(request.metadata())?
        } else {
            request.get_ref().user_id.clone()
        };

        let req = request.into_inner();

        let command = UpdateUserProfileCommand {
            user_id,
            display_name: if req.display_name.is_empty() {
                None
            } else {
                Some(req.display_name)
            },
            avatar_url: if req.avatar_url.is_empty() {
                None
            } else {
                Some(req.avatar_url)
            },
        };

        let user_dto = self.update_user_profile_handler
            .handle(command)
            .await
            .map_err(|e| Self::map_error(e.into()))?;

        Ok(Response::new(UpdateUserProfileResponse {
            user: Some(self.map_user_info(String::new(), user_dto)),
        }))
    }

    async fn list_roles(
        &self,
        request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let req = request.into_inner();
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let response = self.list_roles_handler.handle(ListRolesCommand { limit, offset })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListRolesResponse {
            roles: response.roles.into_iter().map(|r| Role {
                role_id: r.role_id,
                name: r.name,
                description: r.description.unwrap_or_default(),
                created_at: Some(Self::to_timestamp(r.created_at)),
            }).collect(),
            pagination: Some(PageResponse {
                total: response.total,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }

    async fn list_permissions(
        &self,
        request: Request<ListPermissionsRequest>,
    ) -> Result<Response<ListPermissionsResponse>, Status> {
        let req = request.into_inner();
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let response = self.list_permissions_handler.handle(ListPermissionsCommand { limit, offset })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListPermissionsResponse {
            permissions: response.permissions.into_iter().map(|p| Permission {
                permission_id: p.permission_id,
                resource: p.resource,
                actions: vec![p.action],
                scope: String::new(),
                attributes: std::collections::HashMap::new(),
                description: p.description.unwrap_or_default(),
            }).collect(),
            pagination: Some(PageResponse {
                total: response.total,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }

    async fn delete_role(
        &self,
        request: Request<DeleteRoleRequest>,
    ) -> Result<Response<DeleteRoleResponse>, Status> {
        let req = request.into_inner();
        
        self.delete_role_handler.handle(DeleteRoleCommand { role_id: req.role_id })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteRoleResponse { success: true }))
    }

    async fn remove_role_from_user(
        &self,
        request: Request<RemoveRoleFromUserRequest>,
    ) -> Result<Response<RemoveRoleFromUserResponse>, Status> {
        let req = request.into_inner();

        self.remove_role_from_user_handler.handle(RemoveRoleFromUserCommand {
            user_id: req.user_id,
            role_id: req.role_id,
        })
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RemoveRoleFromUserResponse { success: true }))
    }

    async fn remove_permission_from_role(
        &self,
        request: Request<RemovePermissionFromRoleRequest>,
    ) -> Result<Response<RemovePermissionFromRoleResponse>, Status> {
        let req = request.into_inner();

        self.remove_permission_from_role_handler.handle(RemovePermissionFromRoleCommand {
            role_id: req.role_id,
            permission_id: req.permission_id,
        })
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RemovePermissionFromRoleResponse { success: true }))
    }

    async fn get_audit_logs(
        &self,
        request: Request<GetAuditLogsRequest>,
    ) -> Result<Response<GetAuditLogsResponse>, Status> {
        let context_user_id = request.metadata().get("x-user-id").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let context_tenant_id = request.metadata().get("x-tenant-id").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        let req = request.into_inner();
        
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let command = GetAuditLogsCommand {
            user_id: if req.user_id.is_empty() { None } else { Some(req.user_id) },
            tenant_id: if req.tenant_id.is_empty() { None } else { Some(req.tenant_id) },
            action: None,
            resource: None,
            start_time: req.start_time.and_then(|ts| chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)),
            end_time: req.end_time.and_then(|ts| chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)),
            context_user_id,
            context_tenant_id,
            limit,
            offset,
        };

        let response = self.get_audit_logs_handler.handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetAuditLogsResponse {
            logs: response.logs.into_iter().map(|l| AuditLog {
                log_id: l.id,
                user_id: l.user_id,
                tenant_id: l.tenant_id,
                action: l.action,
                resource: l.resource,
                ip_address: l.ip_address,
                user_agent: l.user_agent,
                timestamp: Some(Self::to_timestamp(l.timestamp)),
                details: l.details,
            }).collect(),
            pagination: Some(PageResponse {
                total: response.total as i64,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }

    async fn list_user_sessions(
        &self,
        request: Request<ListUserSessionsRequest>,
    ) -> Result<Response<ListUserSessionsResponse>, Status> {
        let user_id = self.get_user_id_from_metadata(request.metadata())?;

        let response = self.list_user_sessions_handler.handle(ListUserSessionsCommand { user_id: user_id.to_string() })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListUserSessionsResponse {
            sessions: response.sessions.into_iter().map(|s| UserSession {
                session_id: s.session_id,
                device_name: s.device_name,
                ip_address: s.ip_address,
                location: s.location,
                last_active: Some(Self::to_timestamp(s.last_active)),
                created_at: Some(Self::to_timestamp(s.created_at)),
                is_current: false, 
            }).collect(),
        }))
    }

    async fn revoke_session(
        &self,
        request: Request<RevokeSessionRequest>,
    ) -> Result<Response<RevokeSessionResponse>, Status> {
        let user_id = self.get_user_id_from_metadata(request.metadata())?;

        let req = request.into_inner();
        
        self.revoke_session_handler.handle(RevokeSessionCommand {
            user_id: user_id.to_string(),
            session_id: req.session_id,
        })
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RevokeSessionResponse { success: true }))
    }

    async fn enable2_fa(
        &self,
        request: Request<Enable2FaRequest>,
    ) -> Result<Response<Enable2FaResponse>, Status> {
        let user_id = self.get_user_id_from_metadata(request.metadata())?;

        let command = Enable2FACommand {
            user_id: user_id.to_string(),
        };

        let response = self.enable_2fa_handler.handle(command)
            .await
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(Enable2FaResponse {
            secret_key: response.secret_key,
            qr_code_url: response.qr_code_url,
        }))
    }

    async fn verify2_fa_setup(
        &self,
        request: Request<Verify2FaRequest>,
    ) -> Result<Response<Verify2FaResponse>, Status> {
        let user_id = self.get_user_id_from_metadata(request.metadata())?;

        let req = request.into_inner();
        let command = Verify2FASetupCommand {
            user_id: user_id.to_string(),
            code: req.code,
        };

        self.verify_2fa_setup_handler.handle(command)
            .await
            .map_err(|e| Status::invalid_argument(e))?;

        Ok(Response::new(Verify2FaResponse { success: true }))
    }

    async fn verify2_fa_code(
        &self,
        request: Request<Verify2FaCodeRequest>,
    ) -> Result<Response<Verify2FaCodeResponse>, Status> {
        let req = request.into_inner();
        let command = Verify2FACodeCommand {
            temp_token: req.temp_token,
            code: req.code,
        };

        let response = self.verify_2fa_code_handler.handle(command)
            .await
            .map_err(|e| Status::unauthenticated(e))?;

        Ok(Response::new(Verify2FaCodeResponse {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_in: response.expires_in,
        }))
    }





    async fn update_user_status(
        &self,
        request: Request<UpdateUserStatusRequest>,
    ) -> Result<Response<UpdateUserStatusResponse>, Status> {
        let req = request.into_inner();

        let command = UpdateUserStatusCommand {
            user_id: req.user_id,
            is_active: req.is_active,
        };

        self.update_user_status_handler
            .handle(command)
            .await
            .map_err(|e: DomainError| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateUserStatusResponse { success: true }))
    }

    async fn bulk_create_users(
        &self,
        request: Request<BulkCreateUsersRequest>,
    ) -> Result<Response<BulkCreateUsersResponse>, Status> {
        let req = request.into_inner();
        
        let command = BulkCreateUsersCommand {
            users: req.users.into_iter().map(|u| RegisterCommand {
                username: u.username,
                email: u.email,
                password: u.password,
            }).collect(),
        };

        let response = self.bulk_create_users_handler.handle(command).await;

        Ok(Response::new(BulkCreateUsersResponse {
            created_users: response.created_users.into_iter().map(|u| self.map_user_info(String::new(), u)).collect(),
            errors: response.errors,
        }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let command = ListUsersCommand {
            search: if req.search.is_empty() { None } else { Some(req.search) },
            role_id: if req.role_id.is_empty() { None } else { Some(req.role_id) },
            limit,
            offset,
        };

        let response = self.list_users_handler.handle(command)
            .await
            .map_err(|e: RepositoryError| Status::internal(e.to_string()))?;

        Ok(Response::new(ListUsersResponse {
            users: response.users.into_iter().map(|u| self.map_user_info(String::new(), u)).collect(),
            pagination: Some(PageResponse {
                total: response.total as i64,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }

    async fn authorize(
        &self,
        request: Request<AuthorizeRequest>,
    ) -> Result<Response<AuthorizeResponse>, Status> {
        let req = request.into_inner();
        
        let command = AuthorizeCommand {
            user_id: req.user_id,
            client_id: req.client_id,
            redirect_uri: req.redirect_uri,
            scope: req.scope,
            state: req.state,
            response_type: req.response_type,
        };

        let response = self.authorize_handler.handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AuthorizeResponse {
            code: response.code,
            state: response.state,
        }))
    }

    async fn token(
        &self,
        request: Request<TokenRequest>,
    ) -> Result<Response<TokenResponse>, Status> {
        let req = request.into_inner();
        
        let command = OAuth2TokenCommand {
            grant_type: req.grant_type,
            code: if req.code.is_empty() { None } else { Some(req.code) },
            refresh_token: if req.refresh_token.is_empty() { None } else { Some(req.refresh_token) },
            client_id: req.client_id,
            redirect_uri: req.redirect_uri,
            tenant_id: String::new(), // FIXME: OAuth2 Context
        };

        let response = self.oauth2_token_handler.handle(command)
            .await
            .map_err(|e| Status::internal(e))?;

        Ok(Response::new(TokenResponse {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            expires_in: response.expires_in,
            token_type: "Bearer".to_string(),
            scope: String::new(), // FIXME
            id_token: String::new(), // FIXME: OIDC
        }))
    }

    async fn user_info(
        &self,
        request: Request<UserInfoRequest>,
    ) -> Result<Response<UserInfoResponse>, Status> {
        let req = request.into_inner();

        let command = UserInfoCommand {
            access_token: req.access_token,
        };

        let user_dto = self.user_info_handler.handle(command)
            .await
            .map_err(|e| match e {
                DomainError::TokenExpired | DomainError::TokenInvalid | DomainError::TokenRevoked => Status::unauthenticated(e.to_string()),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(UserInfoResponse {
            sub: user_dto.user_id,
            name: user_dto.display_name,
            email: user_dto.email,
            email_verified: user_dto.email_verified,
            picture: user_dto.avatar_url,
        }))
    }

    async fn create_client(
        &self,
        request: Request<CreateClientRequest>,
    ) -> Result<Response<CreateClientResponse>, Status> {
        let req = request.into_inner();
        
        let response = self.create_client_handler.handle(CreateClientCommand {
            name: req.name,
            redirect_uris: req.redirect_uris,
            grant_types: req.grant_types,
            scopes: req.scopes,
        })
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateClientResponse {
            client: Some(Client {
                client_id: response.client.client_id,
                client_secret: response.client_secret_plain,
                name: response.client.name,
                redirect_uris: response.client.redirect_uris,
                grant_types: response.client.grant_types,
                scopes: response.client.scopes,
                created_at: Some(Self::to_timestamp(response.client.created_at)),
            }),
        }))
    }

    async fn list_clients(
        &self,
        request: Request<ListClientsRequest>,
    ) -> Result<Response<ListClientsResponse>, Status> {
        let req = request.into_inner();
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let response = self.list_clients_handler.handle(ListClientsCommand { limit, offset })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ListClientsResponse {
            clients: response.clients.into_iter().map(|c| Client {
                client_id: c.client_id,
                client_secret: String::new(), 
                name: c.name,
                redirect_uris: c.redirect_uris,
                grant_types: c.grant_types,
                scopes: c.scopes,
                created_at: Some(Self::to_timestamp(c.created_at)),
            }).collect(),
            pagination: Some(PageResponse {
                total: response.total,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }

    async fn create_api_key(
        &self,
        request: Request<CreateApiKeyRequest>,
    ) -> Result<Response<CreateApiKeyResponse>, Status> {
        let user_id = request.metadata().get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("User ID not found in metadata"))?
            .to_string();
        let tenant_id = request.metadata().get("x-tenant-id")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string();

        let req = request.into_inner();

        let command = CreateAPIKeyCommand {
            user_id: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            name: req.name.clone(),
            scopes: req.scopes.clone(),
            expires_at: req.expires_at.clone().map(|ts| chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap().with_timezone(&chrono::Utc)),
        };

        let response = self.create_api_key_handler.handle(command)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateApiKeyResponse {
            api_key: response.api_key,
            info: Some(ApiKey {
                key_id: response.key_id,
                name: req.name,
                prefix: response.prefix,
                hashed_key: String::new(), // Don't return hash
                scopes: req.scopes,
                expires_at: req.expires_at,
                created_at: Some(Self::to_timestamp(chrono::Utc::now())),
                created_by_user_id: user_id.to_string(),
            }),
        }))
    }

    async fn list_api_keys(
        &self,
        request: Request<ListApiKeysRequest>,
    ) -> Result<Response<ListApiKeysResponse>, Status> {
        let user_id = self.get_user_id_from_metadata(request.metadata())?;

        let req = request.into_inner();
        
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let command = ListAPIKeysCommand {
            user_id: user_id.to_string(),
            limit,
            offset,
        };
        let response = self.list_api_keys_handler.handle(command)
            .await
            .map_err(|e| Self::map_error(e.into()))?;

        Ok(Response::new(ListApiKeysResponse {
            keys: response.keys.into_iter().map(|k| ApiKey {
                key_id: k.id,
                name: k.name,
                prefix: k.prefix,
                hashed_key: String::new(),
                scopes: k.scopes,
                expires_at: k.expires_at.map(|ts| Self::to_timestamp(ts)),
                created_at: Some(Self::to_timestamp(k.created_at)),
                created_by_user_id: k.user_id.to_string(),
            }).collect(),
            pagination: Some(PageResponse {
                total: response.total as i64,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }

    async fn revoke_api_key(
        &self,
        request: Request<RevokeApiKeyRequest>,
    ) -> Result<Response<RevokeApiKeyResponse>, Status> {
        let req = request.into_inner();
        
        self.revoke_api_key_handler.handle(RevokeAPIKeyCommand { key_id: req.key_id })
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RevokeApiKeyResponse { success: true }))
    }

    async fn sso_login(
        &self,
        request: Request<SsoLoginRequest>,
    ) -> Result<Response<SsoLoginResponse>, Status> {
        let req = request.into_inner();
        let command = SSOLoginCommand {
            provider: req.provider,
            assertion: req.assertion,
        };

        match self.sso_login_handler.handle(command).await {
            Ok((access_token, refresh_token, expires_in)) => {
                // Fetch info for response
                 let user_id = self.token_service.validate_token(&access_token)
                    .map(|claims| claims.sub)
                    .unwrap_or_default();
                    
                let user_info = if !user_id.is_empty() {
                    let user = self.user_repo.find_by_id(&crate::domain::value_objects::UserId::parse(&user_id).unwrap_or_else(|_| crate::domain::value_objects::UserId::new()))
                        .await
                        .unwrap_or(None);
                    user.map(|u| self.map_user_info(String::new(), crate::application::dto::UserDto::from_user(&u)))
                } else {
                    None
                };

                let response = SsoLoginResponse {
                    access_token,
                    refresh_token,
                    expires_in,
                    user: user_info,
                };
                Ok(Response::new(response))
            }
            Err(DomainError::AuthenticationFailed(msg)) => Err(Status::unauthenticated(msg)),
            Err(DomainError::InvalidInput(msg)) => Err(Status::invalid_argument(msg)),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn global_logout(
        &self,
        request: Request<GlobalLogoutRequest>,
    ) -> Result<Response<GlobalLogoutResponse>, Status> {
        let req = request.into_inner();
        
        // 从 access_token 验证用户
        let claims = self.token_service.validate_token(&req.access_token)
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        self.token_service.revoke_all_for_user(&claims.sub).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GlobalLogoutResponse { success: true }))
    }

    async fn create_policy(
        &self,
        request: Request<CreatePolicyRequest>,
    ) -> Result<Response<CreatePolicyResponse>, Status> {
        let req = request.into_inner();
        let tenant_id = "default".to_string(); // Should get from context

        let statements_dto = req.statements.into_iter().map(|s| StatementDto {
            sid: uuid::Uuid::new_v4().to_string(),
            effect: s.effect,
            actions: s.actions,
            resources: s.resources,
        }).collect();

        let command = CreatePolicyCommand {
            name: req.name,
            description: None, // Proto definition might need update, currently using None
            statements: statements_dto,
            tenant_id,
        };

        let policy = self.create_policy_handler.handle(command).await
            .map_err(Self::map_error)?;

        let proto_statements = policy.statements.into_iter().map(|s| Statement {
            effect: match s.effect {
                crate::domain::aggregates::policy::Effect::Allow => "Allow".to_string(),
                crate::domain::aggregates::policy::Effect::Deny => "Deny".to_string(),
            },
            actions: s.actions,
            resources: s.resources,
            conditions: std::collections::HashMap::new(), // Currently no conditions in domain?
        }).collect();

        Ok(Response::new(CreatePolicyResponse {
            policy: Some(crate::proto::Policy {
                policy_id: policy.id,
                name: policy.name,
                version: policy.version,
                statements: proto_statements,
                created_at: Some(Self::to_timestamp(policy.created_at)),
            }),
        }))
    }

    async fn attach_policy_to_role(
        &self,
        request: Request<AttachPolicyToRoleRequest>,
    ) -> Result<Response<AttachPolicyToRoleResponse>, Status> {
        let req = request.into_inner();
        
        self.attach_policy_to_role_handler.handle(AttachPolicyToRoleCommand {
            policy_id: req.policy_id,
            role_id: req.role_id,
        }).await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AttachPolicyToRoleResponse { success: true }))
    }

    async fn attach_policy_to_user(
        &self,
        request: Request<AttachPolicyToUserRequest>,
    ) -> Result<Response<AttachPolicyToUserResponse>, Status> {
        let req = request.into_inner();
        
        self.attach_policy_to_user_handler.handle(AttachPolicyToUserCommand {
            policy_id: req.policy_id,
            user_id: req.user_id,
        }).await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(AttachPolicyToUserResponse { success: true }))
    }

    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "SERVING".to_string(),
        }))
    }

    async fn get_metrics(
        &self,
        _request: Request<GetMetricsRequest>,
    ) -> Result<Response<GetMetricsResponse>, Status> {
        // Collect metrics concurrently
        let total_users_future = self.user_repo.count_all(None, None);
        let active_sessions_future = self.session_repo.count_active();
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);
        let login_attempts_future = self.audit_repo.count_logs(
            None, None, Some("login"), Some(one_hour_ago), None // Corrected call with action filter
        );
        let failed_login_attempts_future = self.audit_repo.count_logs(
            None, None, Some("login_failed"), Some(one_hour_ago), None // Assuming "login_failed" action
        );

        let (total_users, active_sessions, login_attempts, failed_login_attempts) = tokio::join!(
            total_users_future,
            active_sessions_future,
            login_attempts_future,
            failed_login_attempts_future
        );

        Ok(Response::new(GetMetricsResponse {
            metrics: Some(crate::proto::Metrics {
                total_users: total_users.unwrap_or(0),
                active_sessions: active_sessions.unwrap_or(0),
                login_attempts_last_hour: login_attempts.unwrap_or(0),
                failed_logins_last_hour: failed_login_attempts.unwrap_or(0),
            }),
        }))
    }

    async fn social_login(
        &self,
        request: Request<SocialLoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();

        let command = SocialLoginCommand {
            provider: req.provider,
            code: req.code,
            redirect_uri: req.redirect_uri,
        };

        let login_response = self.social_login_handler.handle(command)
            .await
            .map_err(Self::map_error)?;

        Ok(Response::new(LoginResponse {
            access_token: login_response.access_token.unwrap_or_default(),
            refresh_token: login_response.refresh_token.unwrap_or_default(),
            expires_in: login_response.expires_in.unwrap_or_default(),
            user: Some(self.map_user_info(String::new(), login_response.user)),
            requires_2fa: login_response.requires_2fa,
            account_locked: false,
            lock_until: None,
            temp_token: login_response.temp_token.unwrap_or_default(),
        }))
    }

    async fn get_policy(
        &self,
        request: Request<GetPolicyRequest>,
    ) -> Result<Response<GetPolicyResponse>, Status> {
        let req = request.into_inner();

        let policy = self.policy_repo.find_by_id(&req.policy_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Policy not found"))?;

        let statements = policy.statements.into_iter().map(|s| Statement {
            effect: match s.effect {
                crate::domain::aggregates::policy::Effect::Allow => "Allow".to_string(),
                crate::domain::aggregates::policy::Effect::Deny => "Deny".to_string(),
            },
            actions: s.actions,
            resources: s.resources,
            conditions: std::collections::HashMap::new(),
        }).collect();

        Ok(Response::new(GetPolicyResponse {
            policy: Some(crate::proto::Policy {
                policy_id: policy.id,
                name: policy.name,
                version: policy.version,
                statements,
                created_at: Some(Self::to_timestamp(policy.created_at)),
            }),
        }))
    }

    async fn delete_policy(
        &self,
        request: Request<DeletePolicyRequest>,
    ) -> Result<Response<DeletePolicyResponse>, Status> {
        let req = request.into_inner();

        self.policy_repo.delete(&req.policy_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeletePolicyResponse { success: true }))
    }

    async fn list_policies(
        &self,
        request: Request<ListPoliciesRequest>,
    ) -> Result<Response<ListPoliciesResponse>, Status> {
        let req = request.into_inner();
        let limit = req.pagination.as_ref().map(|p| p.page_size as i64).unwrap_or(20);
        let offset = req.pagination.as_ref().map(|p| (p.page as i64) * limit).unwrap_or(0);

        let (policies, total) = self.policy_repo.find_all(limit, offset)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let proto_policies = policies.into_iter().map(|p| {
            let statements = p.statements.into_iter().map(|s| Statement {
                effect: match s.effect {
                    crate::domain::aggregates::policy::Effect::Allow => "Allow".to_string(),
                    crate::domain::aggregates::policy::Effect::Deny => "Deny".to_string(),
                },
                actions: s.actions,
                resources: s.resources,
                conditions: std::collections::HashMap::new(),
            }).collect();

            crate::proto::Policy {
                policy_id: p.id,
                name: p.name,
                version: p.version,
                statements,
                created_at: Some(Self::to_timestamp(p.created_at)),
            }
        }).collect();

        Ok(Response::new(ListPoliciesResponse {
            policies: proto_policies,
            pagination: Some(PageResponse {
                total,
                page: req.pagination.as_ref().map(|p| p.page).unwrap_or(0),
                page_size: limit as i32,
            }),
        }))
    }
}
