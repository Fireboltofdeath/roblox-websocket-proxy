mod api_error;
mod api_response;
mod app_state;
mod config;
mod endpoints;

use std::{env, sync::Arc};

use app_state::AppState;
use axum::{
    routing::{get, post},
    Router,
};
use endpoints::{connect_socket::connect_socket, get_socket::get_socket, send_socket::send_socket};

#[tokio::main]
async fn main() {
    let app_state = AppState {
        sockets: Arc::default(),
    };

    let ip = env::var("IP").unwrap_or("0.0.0.0".to_string());
    let port = env::var("PORT")
        .as_deref()
        .unwrap_or("3000")
        .parse::<u16>()
        .unwrap();

    let app = Router::new()
        .route("/", get(|| async { "Hello from roblox-websocket-proxy!" }))
        .route("/connect", post(connect_socket))
        .route("/:socket_id/get", get(get_socket))
        .route("/:socket_id/send", post(send_socket))
        .with_state(app_state);

    println!("Starting listener on {ip}:{port}");

    let listener = tokio::net::TcpListener::bind((ip, port)).await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
