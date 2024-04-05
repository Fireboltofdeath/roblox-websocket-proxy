use std::{
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
    api_error::ApiError,
    api_response::ApiResponse,
    config::{KEEP_ALIVE, MAX_BATCH_DURATION},
    AppState,
};

#[derive(Deserialize)]
pub struct SocketQuery {
    long: bool,

    /// How many milliseconds to wait after receiving a message.
    /// If not provided, the request will complete immediately.
    batch_ms: Option<u32>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum SocketMessage {
    Content { content: String },
    Close { reason: Option<String> },
}

pub async fn get_socket(
    Query(query): Query<SocketQuery>,
    State(state): State<AppState>,
    Path(socket_id): Path<Uuid>,
) -> Result<ApiResponse<Vec<SocketMessage>>, ApiError> {
    if let Some(socket) = state.find_socket(socket_id).await {
        let is_ready = socket.ready.load(Ordering::Acquire);

        // The socket is already dead and there are no messages to emit.
        if !is_ready && !socket.alive.load(Ordering::Acquire) {
            return Err(ApiError::SocketNotAlive);
        }

        *socket.last_poll.lock().await = Instant::now();

        // The client requested long polling, so we'll wait for messages to be available before sending a response.
        if query.long {
            let notify = socket.notify.clone();

            if !is_ready {
                let result = timeout(KEEP_ALIVE, notify.notified()).await;

                // We passed the keep alive time without receiving any messages, we can immediately return no messages.
                if result.is_err() {
                    return Ok(ApiResponse(Vec::new()));
                }
            }

            if let Some(batch_ms) = query.batch_ms {
                sleep(Duration::from_millis(batch_ms.into()).min(MAX_BATCH_DURATION)).await;
            }
        }

        socket.ready.store(false, Ordering::Release);

        let messages = socket
            .messages
            .lock()
            .await
            .drain(..)
            .filter_map(convert_message)
            .collect::<Vec<SocketMessage>>();

        return Ok(ApiResponse(messages));
    }

    Err(ApiError::SocketNotFound)
}

fn convert_message(message: Message) -> Option<SocketMessage> {
    match message {
        Message::Text(content) => Some(SocketMessage::Content { content }),
        Message::Close(close_frame) => Some(SocketMessage::Close {
            reason: close_frame.map(|v| v.reason.into_owned()),
        }),
        _ => None,
    }
}
