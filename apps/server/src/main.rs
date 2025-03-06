use axum::Json;
use axum::extract::State;
use axum::extract::connect_info::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use axum::{
    Router,
    body::Bytes,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use axum_extra::TypedHeader;
use dotenv::dotenv;
use futures::SinkExt;
use futures::stream::StreamExt;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Deserialize)]
struct BroadcastMessage {
    message: String,
    room: Option<String>,
}

#[derive(Default)]
struct AppState {
    rooms: HashMap<String, Vec<SocketAddr>>,
    clients: HashMap<SocketAddr, futures::channel::mpsc::UnboundedSender<Message>>,
}

impl AppState {
    fn join_room(&mut self, room: String, addr: SocketAddr) {
        let clients = self.rooms.entry(room.clone()).or_default();
        if !clients.contains(&addr) {
            clients.push(addr);
            println!(">>> {addr} joined room: {room}");
        }
    }

    fn leave_room(&mut self, room: String, addr: SocketAddr) {
        if let Some(clients) = self.rooms.get_mut(&room) {
            clients.retain(|a| a != &addr);
        }
        println!(">>> {addr} left room: {room}");
    }

    fn add_client(
        &mut self,
        addr: SocketAddr,
        sender: futures::channel::mpsc::UnboundedSender<Message>,
    ) {
        self.clients.insert(addr, sender);
        println!(">>> Added client: {addr}");
    }

    // fn remove_client(&mut self, addr: &SocketAddr) {
    //     self.clients.remove(addr);
    //     println!(">>> Removed client: {addr}");
    // }

    async fn broadcast(&self, message: String) {
        for (addr, sender) in &self.clients {
            if let Err(e) = sender.unbounded_send(Message::Text(message.clone().into())) {
                println!("Failed to send message to {addr}: {e}");
            }
        }
    }

    async fn broadcast_to_room(&self, room: String, message: String) {
        if let Some(clients) = self.rooms.get(&room) {
            for addr in clients {
                if let Some(sender) = self.clients.get(addr) {
                    if let Err(e) = sender.unbounded_send(Message::Text(message.clone().into())) {
                        println!("Failed to send message to {addr}: {e}");
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    if env::var("BROADCAST_KEY").is_err() {
        panic!("BROADCAST_KEY is not set");
    }

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(Mutex::new(AppState::default()));

    let app = Router::new()
        .route("/wire", any(ws_handler))
        .route("/broadcast", post(broadcast_handler))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

    let listener = tokio::net::TcpListener::bind("[::]:3000").await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

/// Actual websocket state machine (one will be spawned per connection)
async fn handle_socket(mut socket: WebSocket, who: SocketAddr, state: Arc<Mutex<AppState>>) {
    // send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket
        .send(Message::Ping(Bytes::from_static(&[1])))
        .await
        .is_ok()
    {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        // If we can not send messages, there is no way to salvage the statemachine anyway.
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    let (tx, mut rx) = futures::channel::mpsc::unbounded();

    {
        let mut state = state.lock().await;
        state.add_client(who, tx);
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.next().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who, &state).await.is_break() {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    // let mut state = state.lock().await;
    // state.remove_client(&who);
    // // Also clean up from any rooms they were in
    // for (room, clients) in state.rooms.iter_mut() {
    //     if clients.contains(&who) {
    //         state.leave_room(room.clone(), who);
    //     }
    // }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

async fn process_message(
    msg: Message,
    who: SocketAddr,
    state: &Arc<Mutex<AppState>>,
) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            let parts: Vec<&str> = t.split_whitespace().collect();

            match parts.as_slice() {
                ["join", room_name] => {
                    let mut state = state.lock().await;
                    state.join_room(room_name.to_string(), who);
                }
                ["leave", room_name] => {
                    let mut state = state.lock().await;
                    state.leave_room(room_name.to_string(), who);
                }
                _ => {
                    println!(">>> {who} sent str: {t:?}");
                }
            }
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }

        Message::Pong(v) => {
            println!(">>> {who} sent pong with {v:?}");
        }
        // You should never need to manually handle Message::Ping, as axum's websocket library
        // will do so for you automagically by replying with Pong and copying the v according to
        // spec. But if you need the contents of the pings you can see them here.
        Message::Ping(v) => {
            println!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}

async fn broadcast_handler(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
    Json(payload): Json<BroadcastMessage>,
) -> impl IntoResponse {
    let api_key = headers
        .get("x-api-key")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let broadcast_key = env::var("BROADCAST_KEY").unwrap_or_else(|_| {
        panic!("BROADCAST_KEY is not set");
    });

    if api_key != broadcast_key {
        return (StatusCode::UNAUTHORIZED, "Invalid API key").into_response();
    }

    let state = state.lock().await;
    if let Some(room) = payload.room {
        state.broadcast_to_room(room, payload.message).await;
    } else {
        state.broadcast(payload.message).await;
    }

    (StatusCode::OK, "Message broadcasted").into_response()
}
