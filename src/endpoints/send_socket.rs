use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{api_error::ApiError, api_response::ApiResponse, app_state::SocketPacket, AppState};

#[derive(Deserialize)]
pub struct SocketSendBody {
    data: String,
}

pub async fn send_socket(
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
    Json(body): Json<SocketSendBody>,
) -> Result<ApiResponse<()>, ApiError> {
    let socket = state
        .sockets
        .lock()
        .await
        .iter()
        .find(|v| v.id == socket_id)
        .cloned();

    if let Some(socket) = socket {
        socket.sender.send(SocketPacket::Message(body.data)).await?;

        return Ok(ApiResponse(()));
    }

    Err(ApiError::SocketNotFound)
}
