use axum::{http::StatusCode, response::IntoResponse};
use serde::Serialize;
use serde_json::json;

pub struct ApiResponse<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let payload = json!({
            "success": true,
            "result": self.0,
        });

        (StatusCode::OK, axum::Json(payload)).into_response()
    }
}
