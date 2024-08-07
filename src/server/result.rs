use anyhow::Error;
use axum::{
    http::{Response, StatusCode},
    response::IntoResponse,
    Json,
};

use redis::RedisError;
use serde_json::json;

pub type AppResult<T> = Result<T, AppError>;
pub type AppJsonResult<T> = AppResult<Json<T>>;
pub type AppResonseResult<T> = AppResult<Response<T>>;
/// From: https://github.com/Brendonovich/prisma-client-rust/blob/e520c5f6e30c0839d9dbccaa228f3eedbf188b6c/examples/axum-rest/src/routes.rs#L118
pub enum AppError {
    AnyhowError(anyhow::Error),
    RouteError(String),
    RedisError(String),
    Unauthorized(String),
    AuthError(String),
    BadRequest(String),
    NotFound,
    InternalError(String),
    Conflict,
}

impl From<RedisError> for AppError {
    fn from(error: RedisError) -> Self {
        AppError::RedisError(error.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(error: Error) -> Self {
        AppError::AnyhowError(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let (status, error_message) = match self {
            AppError::RouteError(err) => (StatusCode::BAD_REQUEST, err),
            AppError::AnyhowError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::RedisError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Redis Error: {}", err),
            ),
            AppError::Conflict => (StatusCode::CONFLICT, "Conflict".into()),
            AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not Found".into()),
            AppError::InternalError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Internal Server Error: {}", msg),
            ),
            AppError::Unauthorized(msg) => {
                (StatusCode::UNAUTHORIZED, format!("Unauthorized: {}", msg))
            }
        };

        let body = Json(json!({
            "error": error_message
        }));

        (status, body).into_response()
    }
}
