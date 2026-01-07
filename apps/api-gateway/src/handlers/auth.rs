use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
    Extension,
};
use axum::http::HeaderMap;
use crate::state::AppState;
use crate::dto::auth::{
    LoginRequest, LoginResponse, RegisterRequest, RegisterResponse,
    RefreshTokenRequest, RefreshTokenResponse, UserInfo
};

use axum::response::Response;

// 辅助函数：处理 Result 并转换为 Axum Response
fn handle_result<T: serde::Serialize>(result: anyhow::Result<T>) -> Response {
    match result {
        Ok(data) => (StatusCode::OK, Json(data)).into_response(),
        Err(e) => {
            // 简单错误处理，实际项目应根据错误类型返回 400/401/500
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": e.to_string()
            }))).into_response()
        }
    }
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Response {
    let result = state.auth_client.register(payload).await;
    handle_result(result)
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Response {
    let result = state.auth_client.login(payload).await;
    handle_result(result)
}

pub async fn refresh(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Response {
    let result = state.auth_client.refresh_token(payload).await;
    handle_result(result)
}

pub async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    // 从 Header 提取 Bearer Token
    let token = match headers.get("Authorization") {
        Some(value) => {
            let s = value.to_str().unwrap_or("");
            if s.starts_with("Bearer ") {
                &s[7..]
            } else {
                s
            }
        },
        None => return (StatusCode::UNAUTHORIZED, "Missing Authorization header").into_response(),
    };

    let result = state.auth_client.get_user_info(token).await;
    handle_result(result)
}
