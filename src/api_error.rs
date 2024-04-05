use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ApiError {
    ServerError,
    ConnectionError(tokio_tungstenite::tungstenite::Error),
    Raw(u16, String),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::Raw(status_code, _) => {
                StatusCode::from_u16(*status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            ApiError::ConnectionError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::ServerError => "Internal Server Error".to_string(),
            ApiError::Raw(_, message) => message.clone(),
            ApiError::ConnectionError(_) => "WebSocket connection error".to_string(),
        }
    }
}

impl From<tokio_tungstenite::tungstenite::Error> for ApiError {
    fn from(value: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::ConnectionError(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status_code = self.status_code();
        let message = self.message();
        let payload = json!({
            "success": false,
            "error": message,
        });

        (status_code, axum::Json(payload)).into_response()
    }
}
