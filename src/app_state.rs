use std::sync::{atomic::AtomicBool, Arc};

use tokio::sync::{mpsc::Sender, Mutex, Notify};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub sockets: Arc<Mutex<Vec<Arc<Socket>>>>,
}

pub struct Socket {
    pub id: Uuid,
    pub ready: AtomicBool,
    pub messages: Mutex<Vec<Message>>,
    pub sender: Sender<String>,
    pub notify: Arc<Notify>,
}
