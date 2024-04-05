use axum::extract::{Path, State};
use uuid::Uuid;

use crate::{api_error::ApiError, api_response::ApiResponse, app_state::SocketPacket, AppState};

pub async fn close_socket(
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    let socket = state
        .sockets
        .lock()
        .await
        .iter()
        .find(|v| v.id == socket_id)
        .cloned();

    if let Some(socket) = socket {
        socket.sender.send(SocketPacket::Close).await?;

        return Ok(ApiResponse(()));
    }

    Err(ApiError::SocketNotFound)
}
