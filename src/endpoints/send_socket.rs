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

#[derive(Deserialize)]
pub struct SocketSendBody {
    data: String,
}

pub async fn send_socket(
    Authentication: Authentication,
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
    Json(body): Json<SocketSendBody>,
) -> Result<ApiResponse<()>, ApiError> {
    if let Some(socket) = state.find_socket(socket_id).await {
        socket.sender.send(SocketPacket::Message(body.data)).await?;

        return Ok(ApiResponse(()));
    }

    Err(ApiError::SocketNotFound)
}
