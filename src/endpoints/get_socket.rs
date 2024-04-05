use std::{sync::atomic::Ordering, time::Duration};

use axum::extract::{Path, Query, State};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, timeout};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
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
) -> ApiResponse<Vec<SocketMessage>> {
    let socket = state
        .sockets
        .lock()
        .await
        .iter()
        .find(|v| v.id == socket_id)
        .cloned();

    if let Some(socket) = socket {
        // The client requested long polling, so we'll wait for messages to be available before sending a response.
        if query.long {
            let notify = socket.notify.clone();
            let is_ready = socket.ready.load(Ordering::Acquire);

            if !is_ready {
                let result = timeout(KEEP_ALIVE, notify.notified()).await;

                // We passed the keep alive time without receiving any messages, we can immediately return no messages.
                if result.is_err() {
                    return ApiResponse(Vec::new());
                }
            }

            if let Some(batch_ms) = query.batch_ms {
                sleep(Duration::from_millis(batch_ms.into()).min(MAX_BATCH_DURATION)).await;
            }
        }

        socket.ready.store(false, Ordering::Release);

        let messages = socket.messages.lock().await.drain(..).collect::<Vec<_>>();
        return ApiResponse(
            messages
                .into_iter()
                .filter_map(|message| match message {
                    Message::Text(content) => Some(SocketMessage::Content { content }),
                    Message::Close(close_frame) => Some(SocketMessage::Close {
                        reason: close_frame.map(|v| v.reason.into_owned()),
                    }),
                    _ => None,
                })
                .collect(),
        );
    }

    ApiResponse(Vec::new())
}
