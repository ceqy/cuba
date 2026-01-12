use tonic::{Request, Response, Status};
use crate::infrastructure::grpc::iam::v1::auth::auth_service_server::AuthService;
use crate::infrastructure::grpc::iam::v1::auth::*;
use crate::infrastructure::grpc::iam::v1::rbac::rbac_service_server::RbacService;
use crate::infrastructure::grpc::iam::v1::rbac::*;
use crate::infrastructure::grpc::common::v1 as common_v1;
use crate::infrastructure::grpc::iam::v1::common::User as UserProto;
use crate::infrastructure::grpc::iam::v1::common::Role as RoleProto;
use crate::application::commands::RegisterUserCommand;
use crate::application::handlers::RegisterUserHandler;
use cuba_cqrs::CommandHandler;
use std::sync::Arc;

use crate::application::queries::LoginUserQuery;
use crate::application::handlers::LoginUserHandler;
use cuba_cqrs::QueryHandler;
use crate::domain::repositories::{UserRepository, RoleRepository, PermissionRepository};
use crate::domain::aggregates::user::UserId;
use crate::domain::services::TokenService;

pub struct AuthServiceImpl<R: UserRepository, T: TokenService, PR: PermissionRepository> {
    register_handler: Arc<RegisterUserHandler>,
    login_handler: Arc<LoginUserHandler>,
    user_repository: Arc<R>,
    token_service: Arc<T>,
    permission_repository: Arc<PR>,
}

impl<R: UserRepository, T: TokenService, PR: PermissionRepository> AuthServiceImpl<R, T, PR> {
    pub fn new(
        register_handler: Arc<RegisterUserHandler>,
        login_handler: Arc<LoginUserHandler>,
        user_repository: Arc<R>,
        token_service: Arc<T>,
        permission_repository: Arc<PR>,
    ) -> Self {
        Self { register_handler, login_handler, user_repository, token_service, permission_repository }
    }
}

#[tonic::async_trait]
impl<R: UserRepository + 'static, T: TokenService + 'static, PR: PermissionRepository + 'static> AuthService for AuthServiceImpl<R, T, PR> {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let req = request.into_inner();
        let cmd = RegisterUserCommand {
            username: req.username,
            email: req.email,
            password: req.password,
            tenant_id: if req.tenant_id.is_empty() { None } else { Some(req.tenant_id) },
        };

        match self.register_handler.handle(cmd).await {
            Ok(user) => {
                Ok(Response::new(RegisterResponse {
                    user_id: user.id.into_inner().to_string(),
                    username: user.username,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let req = request.into_inner();
        let query = LoginUserQuery {
            username: req.username,
            password: req.password,
            tenant_id: if req.tenant_id.is_empty() { None } else { Some(req.tenant_id) },
        };

        match self.login_handler.handle(query).await {
            Ok(dto) => {
                Ok(Response::new(LoginResponse {
                    access_token: dto.token_pair.access_token,
                    refresh_token: dto.token_pair.refresh_token,
                    expires_in: dto.token_pair.expires_in as i32,
                    token_type: "Bearer".to_string(),
                }))
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Invalid username or password") {
                    Err(Status::unauthenticated(msg))
                } else {
                    Err(Status::internal(msg))
                }
            }
        }
    }

    async fn get_current_user(&self, request: Request<()>) -> Result<Response<UserProto>, Status> {
        let auth_header = request.metadata().get("authorization")
            .ok_or_else(|| Status::unauthenticated("Authorization header required"))?;
        
        let auth_str = auth_header.to_str()
            .map_err(|_| Status::unauthenticated("Invalid header encoding"))?;
        
        let token = auth_str.strip_prefix("Bearer ")
            .ok_or_else(|| Status::unauthenticated("Invalid Bearer token format"))?;
        
        let claims = self.token_service.validate_token(token)
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;
        
        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&claims.sub)
                .map_err(|_| Status::internal("Invalid user_id in token"))?
        );

        match self.user_repository.find_by_id(&user_id).await {
            Ok(Some(user)) => {
                Ok(Response::new(UserProto {
                    user_id: user.id.into_inner().to_string(),
                    username: user.username,
                    email: user.email,
                    tenant_id: user.tenant_id.unwrap_or_default(),
                    is_active: true,
                    created_at: Some(prost_types::Timestamp {
                        seconds: user.created_at.timestamp(),
                        nanos: user.created_at.timestamp_subsec_nanos() as i32,
                    }),
                    roles: user.roles,
                    ..Default::default()
                }))
            }
            Ok(None) => Err(Status::not_found("User not found")),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn change_password(&self, _request: Request<ChangePasswordRequest>) -> Result<Response<ChangePasswordResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
    
    async fn get_perm_codes(&self, request: Request<()>) -> Result<Response<GetPermCodesResponse>, Status> {
        let auth_header = request.metadata().get("authorization")
            .ok_or_else(|| Status::unauthenticated("Authorization header required"))?;
        
        let auth_str = auth_header.to_str()
            .map_err(|_| Status::unauthenticated("Invalid header encoding"))?;
        
        let token = auth_str.strip_prefix("Bearer ")
            .ok_or_else(|| Status::unauthenticated("Invalid Bearer token format"))?;
        
        let claims = self.token_service.validate_token(token)
            .map_err(|e| Status::unauthenticated(format!("Invalid token: {}", e)))?;
        
        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&claims.sub)
                .map_err(|_| Status::internal("Invalid user_id in token"))?
        );

        match self.permission_repository.find_by_user_id(&user_id).await {
            Ok(perms) => {
                Ok(Response::new(GetPermCodesResponse {
                    codes: perms.into_iter().map(|p| p.code).collect(),
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

pub struct RBACServiceImpl<RR: RoleRepository, PR: PermissionRepository> {
    role_repository: Arc<RR>,
    permission_repository: Arc<PR>,
}

impl<RR: RoleRepository, PR: PermissionRepository> RBACServiceImpl<RR, PR> {
    pub fn new(role_repository: Arc<RR>, permission_repository: Arc<PR>) -> Self {
        let _ = role_repository; // Suppress unused warning for now as it will be used for ListRoles
        Self { role_repository, permission_repository }
    }
}

#[tonic::async_trait]
impl<RR: RoleRepository + 'static, PR: PermissionRepository + 'static> RbacService for RBACServiceImpl<RR, PR> {
    async fn create_role(&self, _request: Request<CreateRoleRequest>) -> Result<Response<RoleProto>, Status> {
        Err(Status::unimplemented("Use DB management for now"))
    }

    async fn list_roles(&self, _request: Request<common_v1::PaginationRequest>) -> Result<Response<ListRolesResponse>, Status> {
        Err(Status::unimplemented("Use DB management for now"))
    }

    async fn delete_role(&self, _request: Request<DeleteRoleRequest>) -> Result<Response<common_v1::BatchOperationResult>, Status> {
        Err(Status::unimplemented("Use DB management for now"))
    }

    async fn list_permissions(&self, _request: Request<common_v1::PaginationRequest>) -> Result<Response<ListPermissionsResponse>, Status> {
        Err(Status::unimplemented("Use DB management for now"))
    }

    async fn grant_permissions_to_role(&self, _request: Request<GrantPermissionsRequest>) -> Result<Response<GrantPermissionsResponse>, Status> {
        Err(Status::unimplemented("Use DB management for now"))
    }

    async fn check_permissions(&self, request: Request<CheckPermissionsRequest>) -> Result<Response<CheckPermissionsResponse>, Status> {
        let req = request.into_inner();
        let user_id = UserId::from_uuid(
            uuid::Uuid::parse_str(&req.user_id)
                .map_err(|_| Status::invalid_argument("Invalid user_id"))?
        );

        match self.permission_repository.find_by_user_id(&user_id).await {
            Ok(perms) => {
                let user_codes: std::collections::HashSet<String> = perms.into_iter().map(|p| p.code).collect();
                let mut missing = Vec::new();
                for requested in req.permissions {
                    if !user_codes.contains(&requested) {
                        missing.push(requested);
                    }
                }

                Ok(Response::new(CheckPermissionsResponse {
                    authorized: missing.is_empty(),
                    missing_permissions: missing,
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}
