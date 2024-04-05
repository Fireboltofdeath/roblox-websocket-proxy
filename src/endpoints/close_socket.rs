use axum::extract::{Path, State};
use uuid::Uuid;

use crate::{api_error::ApiError, api_response::ApiResponse, app_state::SocketPacket, AppState};

pub async fn close_socket(
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
) -> Result<ApiResponse<()>, ApiError> {
    if let Some(socket) = state.find_socket(socket_id).await {
        socket.sender.send(SocketPacket::Close).await?;

        return Ok(ApiResponse(()));
    }

    Err(ApiError::SocketNotFound)
}
