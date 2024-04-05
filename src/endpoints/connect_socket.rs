use std::{
    sync::{atomic::Ordering, Arc},
    time::{Duration, Instant},
};

use axum::{extract::State, Json};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{mpsc, Notify},
    time,
};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
    api_error::ApiError,
    api_response::ApiResponse,
    app_state::{AppState, Socket, SocketPacket},
    config::{CLOSED_CONNECTION_EXPIRY, CONNECTION_POLL_TIMEOUT, CONNECTION_TIMEOUT},
};

#[derive(Deserialize)]
pub struct SocketConnectBody {
    url: String,
}

#[derive(Serialize)]
pub struct ConnectSocketResponse {
    socket_id: Uuid,
}

pub async fn connect_socket(
    State(state): State<AppState>,
    Json(query): Json<SocketConnectBody>,
) -> Result<ApiResponse<ConnectSocketResponse>, ApiError> {
    let socket = create_socket(&state, &query.url).await?;

    Ok(ApiResponse(ConnectSocketResponse {
        socket_id: socket.id,
    }))
}

async fn create_socket(app_state: &AppState, url: &str) -> Result<Arc<Socket>, ApiError> {
    let (sender, mut receiver) = mpsc::channel::<SocketPacket>(64);
    let (mut connection, _) = tokio_tungstenite::connect_async(url).await?;
    let notify = Arc::new(Notify::new());
    let socket = Arc::new(Socket::new(notify.clone(), sender));

    tokio::spawn({
        let socket = socket.clone();
        let app_state = app_state.clone();
        async move {
            let mut last_ping = Instant::now();
            let mut heartbeat = time::interval(Duration::from_secs(1));
            heartbeat.set_missed_tick_behavior(time::MissedTickBehavior::Delay);

            loop {
                select! {
                    packet = receiver.recv() => {
                        match packet {
                            Some(SocketPacket::Close) => {
                                connection.close(None).await.ok();
                            }
                            Some(SocketPacket::Message(message)) => {
                                connection.send(Message::Text(message)).await.ok();
                            },
                            _ => {}
                        }
                    },

                    message = connection.next() => {
                        match message {
                            Some(Ok(message)) => {
                                let mut handle = socket.messages.lock().await;
                                handle.push(message);
                                socket.ready.store(true, Ordering::Release);
                                notify.notify_waiters();
                                last_ping = Instant::now();
                            }
                            Some(Err(_)) | None => {
                                break;
                            }
                        };
                    },

                    _ = heartbeat.tick() => {
                        let last_poll = socket.last_poll.lock().await.elapsed();
                        if last_poll > CONNECTION_POLL_TIMEOUT || last_ping.elapsed() > CONNECTION_TIMEOUT {
                            connection.close(None).await.ok();
                        }
                    }
                };
            }

            socket.alive.store(false, Ordering::Release);
            notify.notify_waiters();

            // The connection is dead, but we wait until it expires to remove the socket entry.
            time::sleep(CLOSED_CONNECTION_EXPIRY).await;

            app_state.sockets.lock().await.retain(|v| v.id != socket.id);
        }
    });

    app_state.sockets.lock().await.push(socket.clone());

    Ok(socket)
}
