use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use axum::extract::{Query, State};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::{
    select,
    sync::{mpsc, Mutex, Notify},
};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::{
    api_error::ApiError,
    api_response::ApiResponse,
    app_state::{AppState, Socket},
};

#[derive(Deserialize)]
pub struct SocketConnectQuery {
    url: String,
}

#[derive(Serialize)]
pub struct ConnectSocketResponse {
    socket_id: Uuid,
}

pub async fn connect_socket(
    Query(query): Query<SocketConnectQuery>,
    State(state): State<AppState>,
) -> Result<ApiResponse<ConnectSocketResponse>, ApiError> {
    let socket = create_socket(&state, &query.url).await?;

    Ok(ApiResponse(ConnectSocketResponse {
        socket_id: socket.id,
    }))
}

async fn create_socket(app_state: &AppState, url: &str) -> Result<Arc<Socket>, ApiError> {
    let (sender, mut receiver) = mpsc::channel::<String>(512);
    let (mut connection, _) = tokio_tungstenite::connect_async(url).await?;
    let notify = Arc::new(Notify::new());
    let socket = Arc::new(Socket {
        messages: Mutex::new(Vec::default()),
        notify: notify.clone(),
        ready: AtomicBool::new(false),
        id: Uuid::new_v4(),
        sender,
    });

    tokio::spawn({
        let socket = socket.clone();
        async move {
            loop {
                select! {
                    message = receiver.recv() => {
                        if let Some(message) = message {
                            connection.send(Message::Text(message)).await.ok();
                        }
                    },

                    message = connection.next() => {
                        match message {
                            Some(Ok(message)) => {
                                let mut handle = socket.messages.lock().await;
                                handle.push(message);
                                socket.ready.store(true, Ordering::Release);
                                notify.notify_waiters();
                            }
                            Some(Err(err)) => {
                                println!("{err:?}");
                            }
                            None => {
                                println!("END?");
                            }
                        };
                    }
                };
            }
        }
    });

    app_state.sockets.lock().await.push(socket.clone());

    Ok(socket)
}
