use axum::extract::ws::Message;
use redis::{Commands, RedisError};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use tracing::info;

#[derive(Default)]
pub struct AppState {
    pub rooms: HashMap<String, Vec<SocketAddr>>,
    pub clients: HashMap<SocketAddr, futures::channel::mpsc::UnboundedSender<Message>>,
    redis: Option<redis::Client>,
}

impl AppState {
    pub fn new() -> Self {
        if let Ok(redis_url) = env::var("REDIS_URL") {
            let client = redis::Client::open(redis_url).unwrap();

            return Self {
                rooms: HashMap::new(),
                clients: HashMap::new(),
                redis: Some(client),
            };
        }

        Self::default()
    }

    pub async fn join_room(&mut self, room: String, addr: SocketAddr) {
        let clients = self.rooms.entry(room.clone()).or_default();
        if !clients.contains(&addr) {
            clients.push(addr);
            info!(">>> {addr} joined room: {room}");
        }

        while let Ok(message) = self.get_last_message_from_redis(&room) {
            info!("Replaying message to {room}: {message}");
            self.broadcast_to_room(&room, &message).await;
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

    pub async fn broadcast_to_room(&self, room: &str, message: &str) {
        info!("Broadcasting message to {room}: {message}");

        let clients = match self.rooms.get(room) {
            Some(clients) => clients,
            None => {
                let _ = self.push_message_to_redis(room, message);
                return;
            }
        };

        for addr in clients {
            let sender = match self.clients.get(addr) {
                Some(sender) => sender,
                None => continue,
            };

            if let Err(e) = sender.unbounded_send(Message::Text(message.into())) {
                let _ = self.push_message_to_redis(room, message);
                info!("Failed to send message to {addr}: {e}");
            }
        }
    }

    fn push_message_to_redis(&self, room: &str, message: &str) -> Result<(), RedisError> {
        let redis = self.redis.as_ref().ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::ClientError, "Redis client not configured"))
        })?;

        let mut con = redis.get_connection()?;

        info!("Pushing message to Redis: {}", room);

        let _: () = con.rpush(format!("messages:{}", room), message.to_string())?;
        let _: () = con.expire(format!("messages:{}", room), 60 * 60 * 24)?;

        Ok(())
    }

    fn get_last_message_from_redis(&self, room: &str) -> Result<String, RedisError> {
        let redis = self.redis.as_ref().ok_or_else(|| {
            redis::RedisError::from((redis::ErrorKind::ClientError, "Redis client not configured"))
        })?;

        let mut con = redis.get_connection()?;

        let message = con.lpop(format!("messages:{room}"), None)?;

        Ok(message)
    }
}

#[derive(Deserialize)]
pub struct BroadcastMessage {
    pub message: String,
    pub room: String,
}

#[derive(Deserialize)]
pub struct WsConnectionParams {
    pub room: String,
    pub signature: String,
}
