use axum::{http::StatusCode, response::IntoResponse};
use serde_json::json;
use tokio::sync::mpsc;

#[derive(Debug)]
#[allow(dead_code)]
pub enum ApiError {
    ServerError,
    SocketNotFound,
    SocketNotAlive,
    SocketChannelSendError,
    BadAuthentication,
    NoAuthentication,
    ConnectionError(tokio_tungstenite::tungstenite::Error),
    Raw(u16, String),
}

impl ApiError {
    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::ServerError | ApiError::SocketChannelSendError => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            ApiError::SocketNotFound => StatusCode::NOT_FOUND,
            ApiError::SocketNotAlive | ApiError::ConnectionError(_) => StatusCode::BAD_REQUEST,
            ApiError::BadAuthentication | ApiError::NoAuthentication => StatusCode::UNAUTHORIZED,
            ApiError::Raw(status_code, _) => {
                StatusCode::from_u16(*status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    fn message(&self) -> String {
        match self {
            ApiError::ServerError => "Internal Server Error".to_string(),
            ApiError::SocketNotFound => "Socket not found".to_string(),
            ApiError::SocketNotAlive => "Socket not alive".to_string(),
            ApiError::SocketChannelSendError => "Socket channel send failed".to_string(),
            ApiError::BadAuthentication => "Authentication provided is not sufficient".to_string(),
            ApiError::NoAuthentication => "No authentication was provided".to_string(),
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

impl<T> From<mpsc::error::SendError<T>> for ApiError {
    fn from(_value: mpsc::error::SendError<T>) -> Self {
        Self::SocketChannelSendError
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
