use axum::extract::{Path, Query, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::{api_response::ApiResponse, AppState};

#[derive(Deserialize)]
pub struct SocketSendQuery {
    data: String,
}

pub async fn send_socket(
    Query(query): Query<SocketSendQuery>,
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
) -> ApiResponse<()> {
    let socket = state
        .sockets
        .lock()
        .await
        .iter()
        .find(|v| v.id == socket_id)
        .cloned();

    if let Some(socket) = socket {
        socket.sender.send(query.data).await.ok();
    }

    ApiResponse(())
}
