use axum::Json;
use axum::extract::connect_info::ConnectInfo;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{
    body::Bytes,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures::SinkExt;
use futures::StreamExt;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;
use std::env;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::shared::{AppState, WsConnectionParams};

/// The handler for the HTTP request (this gets called when the HTTP request lands at the start
/// of websocket negotiation). After this completes, the actual switching from HTTP to
/// websocket protocol will occur.
/// This is the last point where we can extract TCP/IP metadata such as IP address of the client
/// as well as things from HTTP headers such as user-agent of the browser etc.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<WsConnectionParams>,
) -> impl IntoResponse {
    let signing_key = env::var("SIGNING_KEY").expect("SIGNING_KEY must be set");
    let mut mac = Hmac::<Sha256>::new_from_slice(signing_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(params.room.as_bytes());
    let expected_signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    if params.signature != expected_signature {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Invalid signature"})),
        )
            .into_response();
    }

    let connection_limit: usize = env::var("CONNECTION_LIMIT")
        .unwrap_or_else(|_| "1000".to_string())
        .parse()
        .unwrap_or(1000);

    if state.lock().await.clients.len() >= connection_limit {
        return (StatusCode::SERVICE_UNAVAILABLE, "Connection limit reached").into_response();
    }

    println!(">>> {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, params.room))
}

/// Actual websocket state machine (one will be spawned per connection)
async fn handle_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    state: Arc<Mutex<AppState>>,
    room: String,
) {
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
        state.join_room(room.clone(), who).await;
    }

    let mut send_task = tokio::spawn(async move {
        while let Some(message) = rx.next().await {
            if sender.send(message).await.is_err() {
                break;
            }
        }
    });

    let room_for_message = room.clone();
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who, room_for_message.clone())
                .await
                .is_break()
            {
                break;
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    let mut state = state.lock().await;
    state.remove_client(&who);
    state.leave_room(room, who);

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

async fn process_message(msg: Message, who: SocketAddr, room: String) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {who} sent string: {t:?}");

            match env::var("MESSAGE_WEBHOOK_URL") {
                Ok(webhook_url) => {
                    println!("Sending message to webhook: {webhook_url}");
                    let client = reqwest::Client::new();

                    let _ = client
                        .post(webhook_url)
                        .json(&json!({
                            "message": t.to_string(),
                            "room": room,
                            "sender": who.to_string(),
                        }))
                        .send()
                        .await;
                }
                _ => {
                    println!("No webhook URL set");
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
