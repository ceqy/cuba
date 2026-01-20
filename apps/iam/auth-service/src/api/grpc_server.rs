use crate::application::{LoginUserHandler, RefreshTokenHandler, RegisterUserHandler};
use crate::domain::repositories::UserRepository;
use crate::infrastructure::grpc::common::v1 as common_proto;
use crate::infrastructure::grpc::iam::auth::v1::auth_service_server::AuthService;
use crate::infrastructure::grpc::iam::auth::v1::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct AuthServiceImpl {
    register_handler: Arc<RegisterUserHandler>,
    login_handler: Arc<LoginUserHandler>,
    refresh_token_handler: Arc<RefreshTokenHandler>,
    user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
}

impl AuthServiceImpl {
    pub fn new(
        register_handler: Arc<RegisterUserHandler>,
        login_handler: Arc<LoginUserHandler>,
        refresh_token_handler: Arc<RefreshTokenHandler>,
        user_repository: Arc<dyn UserRepository<Id = String> + Send + Sync>,
    ) -> Self {
        Self {
            register_handler,
            login_handler,
            refresh_token_handler,
            user_repository,
        }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();

        match self
            .register_handler
            .handle(req.username, req.email, req.password, req.tenant_id)
            .await
        {
            Ok(user) => Ok(Response::new(RegisterResponse {
                user_id: user.id,
                username: user.username,
                status: "ACTIVE".to_string(),
                requires_verification: false,
            })),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn login(
        &self,
        request: Request<LoginRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        let user_agent = request
            .metadata()
            .get("user-agent")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());
        let ip_address = request
            .metadata()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let req = request.into_inner();

        match self
            .login_handler
            .handle(
                req.username,
                req.password,
                req.tenant_id,
                user_agent,
                ip_address,
            )
            .await
        {
            Ok((token, refresh_token, session_id, _user)) => Ok(Response::new(LoginResponse {
                access_token: token,
                refresh_token,
                expires_in: 86400,
                token_type: "Bearer".to_string(),
                session_id,
                locked_until: None,
                password_expires_at: None,
                mfa_required: false,
            })),
            Err(e) => Err(Status::unauthenticated(e.to_string())),
        }
    }

    async fn logout(
        &self,
        _request: Request<LogoutRequest>,
    ) -> Result<Response<LogoutResponse>, Status> {
        Ok(Response::new(LogoutResponse { success: true }))
    }

    async fn refresh_token(
        &self,
        request: Request<RefreshTokenRequest>,
    ) -> Result<Response<RefreshTokenResponse>, Status> {
        let req = request.into_inner();
        match self.refresh_token_handler.handle(req.refresh_token).await {
            Ok((access_token, refresh_token)) => Ok(Response::new(RefreshTokenResponse {
                access_token,
                refresh_token,
                expires_in: 86400,
                token_type: "Bearer".to_string(),
            })),
            Err(e) => Err(Status::unauthenticated(e.to_string())),
        }
    }

    async fn validate_token(
        &self,
        _request: Request<ValidateTokenRequest>,
    ) -> Result<Response<ValidateTokenResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_current_user(
        &self,
        request: Request<()>,
    ) -> Result<Response<GetCurrentUserResponse>, Status> {
        // Extract user_id from metadata (set by Envoy or Auth middleware)
        let user_id = request
            .metadata()
            .get("x-user-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing user_id"))?;

        let user_id_string = user_id.to_string();
        match self.user_repository.find_by_id(&user_id_string).await {
            Ok(Some(user)) => {
                let roles = user
                    .roles
                    .iter()
                    .map(|r: &String| common_proto::Role {
                        role_id: "".to_string(), // we only have role names in user struct currently, simplifying
                        name: r.clone(),
                        description: "".to_string(),
                        parent_id: "".to_string(),
                        tenant_id: "".to_string(),
                        is_immutable: false,
                        created_at: None,
                    })
                    .collect();

                Ok(Response::new(GetCurrentUserResponse {
                    user: Some(common_proto::User {
                        user_id: user.id,
                        username: user.username,
                        email: user.email,
                        tenant_id: user.tenant_id,
                        roles: user.roles,
                        created_at: Some(prost_types::Timestamp::from(
                            std::time::SystemTime::from(user.created_at),
                        )),
                        last_login_at: None,
                        avatar_url: "".to_string(),
                        display_name: "".to_string(),
                        phone: "".to_string(),
                        email_verified: false,
                        is_active: true,
                        attributes: None,
                    }),
                    roles,
                }))
            },
            Ok(None) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    // Stubs for other methods
    async fn change_password(
        &self,
        _request: Request<ChangePasswordRequest>,
    ) -> Result<Response<ChangePasswordResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn update_profile(
        &self,
        _request: Request<UpdateProfileRequest>,
    ) -> Result<Response<UpdateProfileResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn enable2_fa(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Enable2FaResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn verify2_fa(
        &self,
        _request: Request<Verify2FaRequest>,
    ) -> Result<Response<Verify2FaResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn verify2_fa_code(
        &self,
        _request: Request<Verify2FaCodeRequest>,
    ) -> Result<Response<LoginResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_audit_logs(
        &self,
        _request: Request<GetAuditLogsRequest>,
    ) -> Result<Response<GetAuditLogsResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn list_user_sessions(
        &self,
        _request: Request<ListUserSessionsRequest>,
    ) -> Result<Response<ListUserSessionsResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn revoke_session(
        &self,
        _request: Request<RevokeSessionRequest>,
    ) -> Result<Response<RevokeSessionResponse>, Status> {
        Err(Status::unimplemented(""))
    }
    async fn get_perm_codes(
        &self,
        _request: Request<()>,
    ) -> Result<Response<GetPermCodesResponse>, Status> {
        // Logic should look up permissions based on user roles. For MVP Auth Service, returning empty or simple list.
        // Complex permission logic belongs in RBAC service or shared lib.
        Ok(Response::new(GetPermCodesResponse { codes: vec![] }))
    }

    async fn list_users(
        &self,
        request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let req = request.into_inner();
        let pagination = req.pagination.unwrap_or_default();
        let page = if pagination.page < 1 {
            1
        } else {
            pagination.page
        };
        let page_size = if pagination.page_size < 1 {
            10
        } else {
            pagination.page_size
        };
        let offset = (page as i64 - 1) * page_size as i64;

        match self
            .user_repository
            .list_users(offset, page_size as i64)
            .await
        {
            Ok((users, total_count)) => {
                let proto_users = users
                    .into_iter()
                    .map(|u| common_proto::User {
                        user_id: u.id,
                        username: u.username,
                        email: u.email,
                        tenant_id: u.tenant_id,
                        roles: u.roles,
                        created_at: Some(prost_types::Timestamp::from(
                            std::time::SystemTime::from(u.created_at),
                        )),
                        last_login_at: None,
                        avatar_url: "".to_string(),
                        display_name: "".to_string(),
                        phone: "".to_string(),
                        email_verified: false,
                        is_active: true,
                        attributes: None,
                    })
                    .collect();

                Ok(Response::new(ListUsersResponse {
                    users: proto_users,
                    pagination: Some(common_proto::PaginationResponse {
                        current_page: page,
                        page_size,
                        total_items: total_count,
                        total_pages: (total_count as f32 / page_size as f32).ceil() as i32,
                    }),
                }))
            },
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn admin_update_user(
        &self,
        request: Request<AdminUpdateUserRequest>,
    ) -> Result<Response<AdminUpdateUserResponse>, Status> {
        let req = request.into_inner();
        let user_id = req.user_id;

        // 1. Fetch user
        let mut user = match self.user_repository.find_by_id(&user_id).await {
            Ok(Some(u)) => u,
            Ok(None) => return Err(Status::not_found("User not found")),
            Err(e) => return Err(Status::internal(e.to_string())),
        };

        // 2. Update fields
        if !req.email.is_empty() {
            user.email = req.email;
        }
        // Simplified: assuming display_name is not stored in auth service yet or reusing username

        // 3. Password reset if provided
        if !req.password.is_empty() {
            user.password_hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
                .map_err(|e| Status::internal(format!("Failed to hash password: {}", e)))?;
        }

        user.updated_at = chrono::Utc::now();

        // 4. Save
        self.user_repository
            .update(&user)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 5. Return updated user
        Ok(Response::new(AdminUpdateUserResponse {
            user: Some(common_proto::User {
                user_id: user.id,
                username: user.username,
                email: user.email,
                tenant_id: user.tenant_id,
                roles: user.roles,
                created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                    user.created_at,
                ))),
                last_login_at: None,
                avatar_url: "".to_string(),
                display_name: "".to_string(),
                phone: "".to_string(),
                email_verified: false,
                is_active: true,
                attributes: None,
            }),
        }))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();
        self.user_repository
            .delete(&req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteUserResponse { success: true }))
    }
}
