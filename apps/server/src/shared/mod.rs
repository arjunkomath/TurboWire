use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

const DEFAULT_REDIS_STREAM_MAXLEN: usize = 1000;
const DEFAULT_REDIS_STREAM_TTL_SECONDS: usize = 86_400;
const REDIS_STREAM_MESSAGE_FIELD: &str = "message";

#[derive(Default)]
pub struct AppState {
    pub rooms: HashMap<String, Vec<SocketAddr>>,
    pub clients: HashMap<SocketAddr, futures::channel::mpsc::UnboundedSender<Message>>,
    redis: Option<redis::Client>,
    redis_stream_config: RedisStreamConfig,
    room_readers: HashMap<String, Arc<AtomicBool>>,
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
                    error!("Invalid REDIS_URL; Redis Streams disabled: {error}");
                }
            }
        }

        state.redis_stream_config = RedisStreamConfig::from_env();

        state
    }

    pub fn redis_client(&self) -> Option<redis::Client> {
        self.redis.clone()
    }

    pub fn redis_stream_config(&self) -> RedisStreamConfig {
        self.redis_stream_config
    }

    pub fn room_has_clients(&self, room: &str) -> bool {
        self.rooms
            .get(room)
            .is_some_and(|clients| !clients.is_empty())
    }

    pub fn join_room(&mut self, room: String, addr: SocketAddr) -> bool {
        let clients = self.rooms.entry(room.clone()).or_default();
        let was_empty = clients.is_empty();
        if !clients.contains(&addr) {
            clients.push(addr);
            info!(">>> {addr} joined room: {room}");
            return was_empty;
        }

        false
    }

    pub fn leave_room(&mut self, room: String, addr: SocketAddr) -> bool {
        let mut room_is_empty = false;

        if let Some(clients) = self.rooms.get_mut(&room) {
            clients.retain(|a| a != &addr);

            if clients.is_empty() {
                info!("Room {room} is empty, deleting...");
                room_is_empty = true;
            }
        }

        if room_is_empty {
            self.rooms.remove(&room);
            self.stop_room_reader(&room);
        }

        info!(">>> {addr} left room: {room}");
        room_is_empty
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

    pub fn register_room_reader(&mut self, room: &str) -> Option<(redis::Client, Arc<AtomicBool>)> {
        let redis = self.redis_client()?;

        if self.room_readers.contains_key(room) {
            return None;
        }

        let active = Arc::new(AtomicBool::new(true));
        self.room_readers.insert(room.to_string(), active.clone());

        Some((redis, active))
    }

    fn stop_room_reader(&mut self, room: &str) {
        if let Some(active) = self.room_readers.remove(room) {
            active.store(false, Ordering::SeqCst);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RedisStreamConfig {
    pub maxlen: usize,
    pub ttl_seconds: usize,
}

impl Default for RedisStreamConfig {
    fn default() -> Self {
        Self {
            maxlen: DEFAULT_REDIS_STREAM_MAXLEN,
            ttl_seconds: DEFAULT_REDIS_STREAM_TTL_SECONDS,
        }
    }
}

impl RedisStreamConfig {
    fn from_env() -> Self {
        Self::from_raw(
            env::var("REDIS_STREAM_MAXLEN").ok().as_deref(),
            env::var("REDIS_STREAM_TTL_SECONDS").ok().as_deref(),
        )
    }

    fn from_raw(maxlen: Option<&str>, ttl_seconds: Option<&str>) -> Self {
        Self {
            maxlen: parse_positive_usize(maxlen, DEFAULT_REDIS_STREAM_MAXLEN),
            ttl_seconds: parse_positive_usize(ttl_seconds, DEFAULT_REDIS_STREAM_TTL_SECONDS),
        }
    }
}

fn parse_positive_usize(raw: Option<&str>, default: usize) -> usize {
    raw.and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

pub fn redis_room_stream_key(room: &str) -> String {
    format!("turbowire:room:{room}:stream")
}

pub fn append_broadcast_to_redis_stream(
    redis: &redis::Client,
    config: RedisStreamConfig,
    broadcast: &BroadcastMessage,
) -> anyhow::Result<String> {
    let mut con = redis.get_connection()?;
    let key = redis_room_stream_key(&broadcast.room);

    let (stream_id, _): (String, bool) = redis::pipe()
        .atomic()
        .cmd("XADD")
        .arg(&key)
        .arg("MAXLEN")
        .arg("~")
        .arg(config.maxlen)
        .arg("*")
        .arg(REDIS_STREAM_MESSAGE_FIELD)
        .arg(&broadcast.message)
        .cmd("EXPIRE")
        .arg(&key)
        .arg(config.ttl_seconds)
        .query(&mut con)?;

    Ok(stream_id)
}

pub fn latest_redis_room_stream_id(
    redis: &redis::Client,
    room: &str,
) -> anyhow::Result<Option<String>> {
    let mut con = redis.get_connection()?;
    let key = redis_room_stream_key(room);
    let reply: redis::streams::StreamRangeReply = redis::cmd("XREVRANGE")
        .arg(key)
        .arg("+")
        .arg("-")
        .arg("COUNT")
        .arg(1)
        .query(&mut con)?;

    Ok(reply.ids.first().map(|entry| entry.id.clone()))
}

pub async fn initial_redis_room_stream_id(redis: redis::Client, room: String) -> String {
    let room_for_read = room.clone();
    match tokio::task::spawn_blocking(move || latest_redis_room_stream_id(&redis, &room_for_read))
        .await
    {
        Ok(Ok(Some(stream_id))) => stream_id,
        Ok(Ok(None)) => "0-0".to_string(),
        Ok(Err(error)) => {
            warn!("Failed to read Redis stream tail for room {room}: {error}");
            "$".to_string()
        }
        Err(error) => {
            warn!("Redis stream tail task failed for room {room}: {error}");
            "$".to_string()
        }
    }
}

pub fn start_redis_room_reader(
    state: Arc<Mutex<AppState>>,
    room: String,
    redis: redis::Client,
    initial_stream_id: String,
    active: Arc<AtomicBool>,
) {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let delivery_room = room.clone();
    let delivery_state = state.clone();
    let delivery_active = active.clone();

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if !delivery_active.load(Ordering::SeqCst) {
                break;
            }

            let state = delivery_state.lock().await;
            if !state.room_has_clients(&delivery_room) {
                break;
            }
            state.broadcast_to_local_room(&delivery_room, &message);
        }
    });

    tokio::task::spawn_blocking(move || {
        let key = redis_room_stream_key(&room);
        let mut last_id = initial_stream_id;

        while active.load(Ordering::SeqCst) {
            let mut con = match redis.get_connection() {
                Ok(con) => con,
                Err(error) => {
                    error!("Failed to connect to Redis stream for room {room}: {error}");
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };

            info!("Reading Redis stream {key} from {last_id}");

            while active.load(Ordering::SeqCst) {
                let reply: redis::RedisResult<redis::streams::StreamReadReply> =
                    redis::cmd("XREAD")
                        .arg("BLOCK")
                        .arg(1000)
                        .arg("COUNT")
                        .arg(100)
                        .arg("STREAMS")
                        .arg(&key)
                        .arg(&last_id)
                        .query(&mut con);

                let reply = match reply {
                    Ok(reply) => reply,
                    Err(error) => {
                        error!("Failed to read Redis stream {key}: {error}");
                        break;
                    }
                };

                for stream_key in reply.keys {
                    for entry in stream_key.ids {
                        last_id = entry.id.clone();
                        let Some(message) = entry.get::<String>(REDIS_STREAM_MESSAGE_FIELD) else {
                            warn!(
                                "Redis stream entry {last_id} in {key} is missing the message field"
                            );
                            continue;
                        };

                        if tx.send(message).is_err() {
                            return;
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(250));
        }

        info!("Stopped Redis stream reader for room {room}");
    });
}

#[derive(Clone, Deserialize, Serialize)]
pub struct BroadcastMessage {
    pub message: String,
    pub room: String,
}

#[derive(Deserialize)]
pub struct WsConnectionParams {
    pub room: String,
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn redis_room_stream_key_uses_room_name() {
        assert_eq!(
            redis_room_stream_key("notifications_user_123"),
            "turbowire:room:notifications_user_123:stream"
        );
    }

    #[test]
    fn redis_stream_config_uses_defaults() {
        assert_eq!(
            RedisStreamConfig::from_raw(None, None),
            RedisStreamConfig {
                maxlen: 1000,
                ttl_seconds: 86_400,
            }
        );
    }

    #[test]
    fn redis_stream_config_accepts_positive_overrides() {
        assert_eq!(
            RedisStreamConfig::from_raw(Some("250"), Some("60")),
            RedisStreamConfig {
                maxlen: 250,
                ttl_seconds: 60,
            }
        );
    }

    #[test]
    fn redis_stream_config_rejects_invalid_overrides() {
        assert_eq!(
            RedisStreamConfig::from_raw(Some("0"), Some("not-a-number")),
            RedisStreamConfig::default()
        );
    }

    #[test]
    fn broadcast_to_local_room_sends_to_joined_clients() {
        let mut state = AppState::default();
        let addr = "127.0.0.1:1234".parse().unwrap();
        let (tx, mut rx) = futures::channel::mpsc::unbounded();

        state.add_client(addr, tx);
        assert!(state.join_room("room-a".to_string(), addr));

        assert_eq!(state.broadcast_to_local_room("room-a", "hello"), 1);

        let message = futures::executor::block_on(rx.next()).unwrap();
        match message {
            Message::Text(text) => assert_eq!(text.as_str(), "hello"),
            _ => panic!("expected text message"),
        }
    }

    #[test]
    fn redis_stream_write_smoke() -> anyhow::Result<()> {
        let Ok(redis_url) = env::var("TURBOWIRE_TEST_REDIS_URL") else {
            return Ok(());
        };

        let redis = redis::Client::open(redis_url)?;
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let room = format!("test-{suffix}");
        let key = redis_room_stream_key(&room);
        let broadcast = BroadcastMessage {
            room,
            message: "hello".to_string(),
        };
        let config = RedisStreamConfig {
            maxlen: 2,
            ttl_seconds: 60,
        };

        let stream_id = append_broadcast_to_redis_stream(&redis, config, &broadcast)?;
        assert!(!stream_id.is_empty());

        let mut con = redis.get_connection()?;
        let len: usize = redis::cmd("XLEN").arg(&key).query(&mut con)?;
        let ttl: i64 = redis::cmd("TTL").arg(&key).query(&mut con)?;

        redis::cmd("DEL").arg(&key).query::<()>(&mut con)?;

        assert!(len >= 1);
        assert!(ttl > 0 && ttl <= 60);

        Ok(())
    }
}
