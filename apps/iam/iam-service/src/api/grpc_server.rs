use tonic::{Request, Response, Status};
use crate::infrastructure::grpc::iam::v1::auth_service_server::AuthService;
use crate::infrastructure::grpc::iam::v1::*;
use crate::application::commands::RegisterUserCommand;
use crate::application::handlers::RegisterUserHandler;
use cuba_cqrs::CommandHandler;
use std::sync::Arc;

pub struct AuthServiceImpl {
    register_handler: Arc<RegisterUserHandler>,
}

impl AuthServiceImpl {
    pub fn new(register_handler: Arc<RegisterUserHandler>) -> Self {
        Self { register_handler }
    }
}

#[tonic::async_trait]
impl AuthService for AuthServiceImpl {
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
                    user: Some(UserInfo {
                        user_id: user.id.into_inner().to_string(),
                        username: user.username,
                        email: user.email,
                        tenant_id: user.tenant_id.unwrap_or_default(),
                        is_active: true,
                        created_at: Some(prost_types::Timestamp {
                            seconds: user.created_at.timestamp(),
                            nanos: user.created_at.timestamp_subsec_nanos() as i32,
                        }),
                        updated_at: Some(prost_types::Timestamp {
                            seconds: user.updated_at.timestamp(),
                            nanos: user.updated_at.timestamp_subsec_nanos() as i32,
                        }),
                        ..Default::default()
                    })
                }))
            }
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }

    async fn login(&self, _request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn logout(&self, _request: Request<LogoutRequest>) -> Result<Response<LogoutResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn refresh_token(&self, _request: Request<RefreshTokenRequest>) -> Result<Response<RefreshTokenResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn validate_token(&self, _request: Request<ValidateTokenRequest>) -> Result<Response<ValidateTokenResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_user_info(&self, _request: Request<GetUserInfoRequest>) -> Result<Response<GetUserInfoResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn create_role(&self, _request: Request<CreateRoleRequest>) -> Result<Response<CreateRoleResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn assign_role_to_user(&self, _request: Request<AssignRoleToUserRequest>) -> Result<Response<AssignRoleToUserResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn add_permission_to_role(&self, _request: Request<AddPermissionToRoleRequest>) -> Result<Response<AddPermissionToRoleResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_user_permissions(&self, _request: Request<GetUserPermissionsRequest>) -> Result<Response<GetUserPermissionsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn change_password(&self, _request: Request<ChangePasswordRequest>) -> Result<Response<ChangePasswordResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn reset_password(&self, _request: Request<ResetPasswordRequest>) -> Result<Response<ResetPasswordResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn send_password_reset_token(&self, _request: Request<SendPasswordResetTokenRequest>) -> Result<Response<SendPasswordResetTokenResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn verify_email(&self, _request: Request<VerifyEmailRequest>) -> Result<Response<VerifyEmailResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn update_user_profile(&self, _request: Request<UpdateUserProfileRequest>) -> Result<Response<UpdateUserProfileResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_roles(&self, _request: Request<ListRolesRequest>) -> Result<Response<ListRolesResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_permissions(&self, _request: Request<ListPermissionsRequest>) -> Result<Response<ListPermissionsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn delete_role(&self, _request: Request<DeleteRoleRequest>) -> Result<Response<DeleteRoleResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn remove_role_from_user(&self, _request: Request<RemoveRoleFromUserRequest>) -> Result<Response<RemoveRoleFromUserResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn remove_permission_from_role(&self, _request: Request<RemovePermissionFromRoleRequest>) -> Result<Response<RemovePermissionFromRoleResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_audit_logs(&self, _request: Request<GetAuditLogsRequest>) -> Result<Response<GetAuditLogsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_user_sessions(&self, _request: Request<ListUserSessionsRequest>) -> Result<Response<ListUserSessionsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn revoke_session(&self, _request: Request<RevokeSessionRequest>) -> Result<Response<RevokeSessionResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn enable2_fa(&self, _request: Request<Enable2FaRequest>) -> Result<Response<Enable2FaResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn verify2_fa_setup(&self, _request: Request<Verify2FaRequest>) -> Result<Response<Verify2FaResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn verify2_fa_code(&self, _request: Request<Verify2FaCodeRequest>) -> Result<Response<Verify2FaCodeResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn update_user_status(&self, _request: Request<UpdateUserStatusRequest>) -> Result<Response<UpdateUserStatusResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn bulk_create_users(&self, _request: Request<BulkCreateUsersRequest>) -> Result<Response<BulkCreateUsersResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_users(&self, _request: Request<ListUsersRequest>) -> Result<Response<ListUsersResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn authorize(&self, _request: Request<AuthorizeRequest>) -> Result<Response<AuthorizeResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn token(&self, _request: Request<TokenRequest>) -> Result<Response<TokenResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn user_info(&self, _request: Request<UserInfoRequest>) -> Result<Response<UserInfoResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn create_client(&self, _request: Request<CreateClientRequest>) -> Result<Response<CreateClientResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_clients(&self, _request: Request<ListClientsRequest>) -> Result<Response<ListClientsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn create_api_key(&self, _request: Request<CreateApiKeyRequest>) -> Result<Response<CreateApiKeyResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_api_keys(&self, _request: Request<ListApiKeysRequest>) -> Result<Response<ListApiKeysResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn revoke_api_key(&self, _request: Request<RevokeApiKeyRequest>) -> Result<Response<RevokeApiKeyResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn sso_login(&self, _request: Request<SsoLoginRequest>) -> Result<Response<SsoLoginResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn global_logout(&self, _request: Request<GlobalLogoutRequest>) -> Result<Response<GlobalLogoutResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn create_policy(&self, _request: Request<CreatePolicyRequest>) -> Result<Response<CreatePolicyResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn attach_policy_to_role(&self, _request: Request<AttachPolicyToRoleRequest>) -> Result<Response<AttachPolicyToRoleResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn attach_policy_to_user(&self, _request: Request<AttachPolicyToUserRequest>) -> Result<Response<AttachPolicyToUserResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn get_policy(&self, _request: Request<GetPolicyRequest>) -> Result<Response<GetPolicyResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn delete_policy(&self, _request: Request<DeletePolicyRequest>) -> Result<Response<DeletePolicyResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn list_policies(&self, _request: Request<ListPoliciesRequest>) -> Result<Response<ListPoliciesResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn health_check(&self, _request: Request<HealthCheckRequest>) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: "SERVING".to_string(),
        }))
    }

    async fn get_metrics(&self, _request: Request<GetMetricsRequest>) -> Result<Response<GetMetricsResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }

    async fn social_login(&self, _request: Request<SocialLoginRequest>) -> Result<Response<LoginResponse>, Status> {
        Err(Status::unimplemented("Not implemented"))
    }
}
