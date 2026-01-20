use crate::domain::repositories::{PermissionRepository, RoleRepository};
use crate::infrastructure::grpc::common::v1 as common_proto;
use crate::infrastructure::grpc::iam::rbac::v1::rbac_service_server::RbacService;
use crate::infrastructure::grpc::iam::rbac::v1::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RBACServiceImpl<R: RoleRepository, P: PermissionRepository> {
    role_repository: Arc<R>,
    permission_repository: Arc<P>,
}

impl<R: RoleRepository, P: PermissionRepository> RBACServiceImpl<R, P> {
    pub fn new(role_repository: Arc<R>, permission_repository: Arc<P>) -> Self {
        Self {
            role_repository,
            permission_repository,
        }
    }
}

#[tonic::async_trait]
impl<R: RoleRepository + 'static, P: PermissionRepository + 'static> RbacService
    for RBACServiceImpl<R, P>
{
    async fn create_role(
        &self,
        request: Request<CreateRoleRequest>,
    ) -> Result<Response<common_proto::Role>, Status> {
        let req = request.into_inner();
        let role = crate::domain::Role {
            id: uuid::Uuid::new_v4().to_string(),
            name: req.name,
            description: req.description,
            parent_id: if req.parent_id.is_empty() {
                None
            } else {
                Some(req.parent_id)
            },
            tenant_id: req.tenant_id,
            is_immutable: false,
            created_at: chrono::Utc::now(),
        };
        self.role_repository
            .save(&role)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_proto::Role {
            role_id: role.id,
            name: role.name,
            description: role.description,
            parent_id: role.parent_id.unwrap_or_default(),
            tenant_id: role.tenant_id,
            is_immutable: role.is_immutable,
            created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                role.created_at,
            ))),
        }))
    }

    async fn update_role(
        &self,
        request: Request<UpdateRoleRequest>,
    ) -> Result<Response<common_proto::Role>, Status> {
        let req = request.into_inner();
        let role_id = req.role_id;

        let mut existing = self
            .role_repository
            .find_by_id(&role_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Role not found"))?;

        if existing.is_immutable {
            return Err(Status::permission_denied("Cannot update immutable role"));
        }

        if let Some(role_update) = req.role {
            existing.name = role_update.name;
            existing.description = role_update.description;
            existing.parent_id = if role_update.parent_id.is_empty() {
                None
            } else {
                Some(role_update.parent_id)
            };
            // Note: tenant_id and is_immutable usually shouldn't be updated via this simple path
        }

        self.role_repository
            .save(&existing)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(common_proto::Role {
            role_id: existing.id,
            name: existing.name,
            description: existing.description,
            parent_id: existing.parent_id.unwrap_or_default(),
            tenant_id: existing.tenant_id,
            is_immutable: existing.is_immutable,
            created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                existing.created_at,
            ))),
        }))
    }

    async fn list_roles(
        &self,
        _request: Request<ListRolesRequest>,
    ) -> Result<Response<ListRolesResponse>, Status> {
        let roles = self
            .role_repository
            .find_all()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListRolesResponse {
            roles: roles
                .into_iter()
                .map(|r| common_proto::Role {
                    role_id: r.id,
                    name: r.name,
                    description: r.description,
                    parent_id: r.parent_id.unwrap_or_default(),
                    tenant_id: r.tenant_id,
                    is_immutable: r.is_immutable,
                    created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                        r.created_at,
                    ))),
                })
                .collect(),
            pagination: None,
        }))
    }

    async fn delete_role(
        &self,
        request: Request<DeleteRoleRequest>,
    ) -> Result<Response<common_proto::BatchOperationResult>, Status> {
        let role_id = request.into_inner().role_id;
        let existing = self
            .role_repository
            .find_by_id(&role_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("Role not found"))?;

        if existing.is_immutable {
            return Err(Status::permission_denied("Cannot delete immutable role"));
        }

        self.role_repository
            .delete(&role_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(common_proto::BatchOperationResult {
            success_count: 1,
            failure_count: 0,
            errors: vec![],
        }))
    }

    async fn list_permissions(
        &self,
        _request: Request<common_proto::PaginationRequest>,
    ) -> Result<Response<ListPermissionsResponse>, Status> {
        let perms = self
            .permission_repository
            .find_all()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(ListPermissionsResponse {
            permissions: perms
                .into_iter()
                .map(|p| common_proto::Permission {
                    permission_id: p.id,
                    code: p.code,
                    resource: p.resource,
                    action: p.action,
                    description: p.description,
                    created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                        p.created_at,
                    ))),
                    conditions: None,
                    effect: 0,
                })
                .collect(),
            pagination: None,
        }))
    }

    async fn grant_permissions_to_role(
        &self,
        request: Request<GrantPermissionsRequest>,
    ) -> Result<Response<GrantPermissionsResponse>, Status> {
        let req = request.into_inner();
        self.role_repository
            .grant_permissions(&req.role_id, &req.permission_ids)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GrantPermissionsResponse { success: true }))
    }

    async fn assign_role_to_user(
        &self,
        request: Request<AssignRoleToUserRequest>,
    ) -> Result<Response<AssignRoleToUserResponse>, Status> {
        let req = request.into_inner();
        self.role_repository
            .assign_to_user(&req.user_id, &req.role_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(AssignRoleToUserResponse { success: true }))
    }

    async fn remove_role_from_user(
        &self,
        request: Request<RemoveRoleFromUserRequest>,
    ) -> Result<Response<RemoveRoleFromUserResponse>, Status> {
        let req = request.into_inner();
        self.role_repository
            .remove_from_user(&req.user_id, &req.role_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(RemoveRoleFromUserResponse { success: true }))
    }

    async fn check_permissions(
        &self,
        request: Request<CheckPermissionsRequest>,
    ) -> Result<Response<CheckPermissionsResponse>, Status> {
        let req = request.into_inner();
        let perms = self
            .permission_repository
            .find_by_user_id(&req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        let codes: std::collections::HashSet<String> = perms.into_iter().map(|p| p.code).collect();
        let mut results = std::collections::HashMap::new();
        let mut missing = Vec::new();
        for p in &req.permissions {
            let has = codes.contains(p);
            results.insert(p.clone(), has);
            if !has {
                missing.push(p.clone());
            }
        }
        Ok(Response::new(CheckPermissionsResponse {
            authorized: missing.is_empty(),
            permission_results: results,
            missing_permissions: missing,
        }))
    }

    async fn get_user_roles(
        &self,
        request: Request<GetUserRolesRequest>,
    ) -> Result<Response<GetUserRolesResponse>, Status> {
        let req = request.into_inner();
        let roles = self
            .role_repository
            .find_by_user_id(&req.user_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetUserRolesResponse {
            roles: roles
                .into_iter()
                .map(|r| common_proto::Role {
                    role_id: r.id,
                    name: r.name,
                    description: r.description,
                    parent_id: r.parent_id.unwrap_or_default(),
                    tenant_id: r.tenant_id,
                    is_immutable: r.is_immutable,
                    created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::from(
                        r.created_at,
                    ))),
                })
                .collect(),
        }))
    }
}
