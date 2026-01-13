use crate::infrastructure::grpc::iam::rbac::v1::rbac_service_client::RbacServiceClient;
use crate::infrastructure::grpc::iam::rbac::v1::CheckPermissionsRequest;
use crate::infrastructure::grpc::common::v1::PaginationRequest;
use tonic::transport::Channel;
use std::sync::Arc;

pub struct RbacClient {
    client: RbacServiceClient<Channel>,
}

impl RbacClient {
    pub async fn new(address: String) -> Result<Self, anyhow::Error> {
        let client = RbacServiceClient::connect(address).await?;
        Ok(Self { client })
    }

    pub async fn get_user_roles(&self, user_id: &str) -> Result<Vec<String>, anyhow::Error> {
        let mut client = self.client.clone();
        let request = tonic::Request::new(crate::infrastructure::grpc::iam::rbac::v1::GetUserRolesRequest {
            user_id: user_id.to_string(),
        });

        let response = client.get_user_roles(request).await?.into_inner();
        let roles = response.roles.into_iter().map(|r| r.name).collect();
        
        Ok(roles)
    }
}
