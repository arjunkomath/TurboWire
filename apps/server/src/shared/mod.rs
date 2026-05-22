use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info};

const REDIS_BROADCAST_CHANNEL: &str = "turbowire:broadcasts";

pub struct AppState {
    pub rooms: HashMap<String, Vec<SocketAddr>>,
    pub clients: HashMap<SocketAddr, futures::channel::mpsc::UnboundedSender<Message>>,
    instance_id: String,
    redis: Option<redis::Client>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            rooms: HashMap::new(),
            clients: HashMap::new(),
            instance_id: uuid::Builder::from_random_bytes(rand::random())
                .into_uuid()
                .to_string(),
            redis: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        let mut state = Self::default();

        if let Ok(redis_url) = env::var("REDIS_URL") {
            match redis::Client::open(redis_url) {
                Ok(client) => {
                    state.redis = Some(client);
                }
                Err(error) => {
                    error!("Invalid REDIS_URL; Redis Pub/Sub disabled: {error}");
                }
            }
        }

        state
    }

    pub fn instance_id(&self) -> String {
        self.instance_id.clone()
    }

    pub fn redis_client(&self) -> Option<redis::Client> {
        self.redis.clone()
    }

    pub fn join_room(&mut self, room: String, addr: SocketAddr) {
        let clients = self.rooms.entry(room.clone()).or_default();
        if !clients.contains(&addr) {
            clients.push(addr);
            info!(">>> {addr} joined room: {room}");
        }
    }

    pub fn leave_room(&mut self, room: String, addr: SocketAddr) {
        if let Some(clients) = self.rooms.get_mut(&room) {
            clients.retain(|a| a != &addr);

            if clients.is_empty() {
                info!("Room {room} is empty, deleting...");
                self.rooms.remove(&room);
            }
        }
        info!(">>> {addr} left room: {room}");
    }

    pub fn add_client(
        &mut self,
        addr: SocketAddr,
        sender: futures::channel::mpsc::UnboundedSender<Message>,
    ) {
        self.clients.insert(addr, sender);
        info!(">>> Added client: {addr}");
    }

    pub fn remove_client(&mut self, addr: &SocketAddr) {
        self.clients.remove(addr);
        info!(">>> Removed client: {addr}");
    }

    pub fn broadcast_to_local_room(&self, room: &str, message: &str) -> usize {
        info!("Broadcasting message to {room}: {message}");

        let clients = match self.rooms.get(room) {
            Some(clients) => clients,
            None => return 0,
        };

        let mut delivered = 0;

        for addr in clients {
            let sender = match self.clients.get(addr) {
                Some(sender) => sender,
                None => continue,
            };

            match sender.unbounded_send(Message::Text(message.into())) {
                Ok(()) => delivered += 1,
                Err(e) => info!("Failed to send message to {addr}: {e}"),
            }
        }

        delivered
    }
}

pub async fn start_redis_broadcast_subscriber(state: Arc<Mutex<AppState>>) {
    let (redis, instance_id) = {
        let state = state.lock().await;
        (state.redis_client(), state.instance_id())
    };

    let Some(redis) = redis else {
        return;
    };

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<BroadcastMessage>();

    tokio::task::spawn_blocking(move || {
        loop {
            let mut con = match redis.get_connection() {
                Ok(con) => con,
                Err(error) => {
                    error!("Failed to connect to Redis for Pub/Sub: {error}");
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };

            let mut pubsub = con.as_pubsub();

            if let Err(error) = pubsub.subscribe(REDIS_BROADCAST_CHANNEL) {
                error!("Failed to subscribe to Redis channel {REDIS_BROADCAST_CHANNEL}: {error}");
                std::thread::sleep(Duration::from_secs(1));
                continue;
            }

            info!("Subscribed to Redis channel {REDIS_BROADCAST_CHANNEL}");

            loop {
                let message = match pubsub.get_message() {
                    Ok(message) => message,
                    Err(error) => {
                        error!("Failed to receive Redis Pub/Sub message: {error}");
                        break;
                    }
                };

                let payload = match message.get_payload::<String>() {
                    Ok(payload) => payload,
                    Err(error) => {
                        error!("Failed to read Redis Pub/Sub payload: {error}");
                        continue;
                    }
                };

                let broadcast = match serde_json::from_str::<RedisBroadcast>(&payload) {
                    Ok(broadcast) => broadcast,
                    Err(error) => {
                        error!("Failed to parse Redis Pub/Sub payload: {error}");
                        continue;
                    }
                };

                if broadcast.origin_id.as_deref() == Some(instance_id.as_str()) {
                    continue;
                }

                if tx
                    .send(BroadcastMessage {
                        room: broadcast.room,
                        message: broadcast.message,
                    })
                    .is_err()
                {
                    return;
                }
            }

            std::thread::sleep(Duration::from_secs(1));
        }
    });

    tokio::spawn(async move {
        while let Some(broadcast) = rx.recv().await {
            let state = state.lock().await;
            state.broadcast_to_local_room(&broadcast.room, &broadcast.message);
        }
    });
}

pub fn publish_broadcast_to_redis(
    redis: &redis::Client,
    origin_id: &str,
    broadcast: &BroadcastMessage,
) -> anyhow::Result<usize> {
    let mut con = redis.get_connection()?;
    let message = serde_json::to_string(&RedisBroadcast {
        origin_id: Some(origin_id.to_string()),
        room: broadcast.room.clone(),
        message: broadcast.message.clone(),
    })?;
    let subscribers = redis::cmd("PUBLISH")
        .arg(REDIS_BROADCAST_CHANNEL)
        .arg(message)
        .query::<usize>(&mut con)?;

    Ok(subscribers)
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BroadcastMessage {
    pub message: String,
    pub room: String,
}

#[derive(Deserialize, Serialize)]
struct RedisBroadcast {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    origin_id: Option<String>,
    message: String,
    room: String,
}

#[derive(Deserialize)]
pub struct WsConnectionParams {
    pub room: String,
    pub signature: String,
}
