use crate::clients::auth::AuthClient;

#[derive(Clone)]
pub struct AppState {
    pub auth_client: AuthClient,
}
