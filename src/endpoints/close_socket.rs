use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    api_error::ApiError, api_response::ApiResponse, app_state::SocketPacket,
    authentication::Authentication, AppState,
};

/// The `#[serde(default)]` isn't normally required, but it must be explicitly provided because of Axum.
#[derive(Deserialize)]
pub struct SocketSendBody {
    #[serde(default)]
    code: Option<u16>,

    #[serde(default)]
    reason: Option<String>,
}

pub async fn close_socket(
    Authentication: Authentication,
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
    Json(body): Json<SocketSendBody>,
) -> Result<ApiResponse<()>, ApiError> {
    if let Some(socket) = state.find_socket(socket_id).await {
        let packet = SocketPacket::Close(body.code.map(Into::into), body.reason.map(Into::into));
        socket.sender.send(packet).await?;

        return Ok(ApiResponse(()));
    }

    Err(ApiError::SocketNotFound)
}
