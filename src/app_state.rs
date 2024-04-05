use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};

use tokio::sync::{mpsc::Sender, Mutex, Notify};
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub sockets: Arc<Mutex<Vec<Arc<Socket>>>>,
}

pub enum SocketPacket {
    Close,
    Message(String),
}

pub struct Socket {
    pub id: Uuid,
    pub ready: AtomicBool,
    pub alive: AtomicBool,
    pub messages: Mutex<Vec<Message>>,
    pub sender: Sender<SocketPacket>,
    pub notify: Arc<Notify>,
    pub last_poll: Mutex<Instant>,
}

impl Socket {
    pub fn new(notify: Arc<Notify>, sender: Sender<SocketPacket>) -> Self {
        Self {
            id: Uuid::new_v4(),
            ready: AtomicBool::new(false),
            alive: AtomicBool::new(true),
            messages: Mutex::new(Vec::default()),
            last_poll: Mutex::new(Instant::now()),
            notify,
            sender,
        }
    }
}
