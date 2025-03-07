use axum::extract::ws::Message;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;

#[derive(Default)]
pub struct AppState {
    pub rooms: HashMap<String, Vec<SocketAddr>>,
    pub clients: HashMap<SocketAddr, futures::channel::mpsc::UnboundedSender<Message>>,
}

impl AppState {
    pub fn join_room(&mut self, room: String, addr: SocketAddr) {
        let clients = self.rooms.entry(room.clone()).or_default();
        if !clients.contains(&addr) {
            clients.push(addr);
            println!(">>> {addr} joined room: {room}");
        }
    }

    pub fn leave_room(&mut self, room: String, addr: SocketAddr) {
        if let Some(clients) = self.rooms.get_mut(&room) {
            clients.retain(|a| a != &addr);
        }
        println!(">>> {addr} left room: {room}");
    }

    pub fn add_client(
        &mut self,
        addr: SocketAddr,
        sender: futures::channel::mpsc::UnboundedSender<Message>,
    ) {
        self.clients.insert(addr, sender);
        println!(">>> Added client: {addr}");
    }

    pub fn remove_client(&mut self, addr: &SocketAddr) {
        self.clients.remove(addr);
        println!(">>> Removed client: {addr}");
    }

    pub async fn broadcast_to_room(&self, room: String, message: String) {
        let clients = match self.rooms.get(&room) {
            Some(clients) => clients,
            None => return,
        };

        for addr in clients {
            let sender = match self.clients.get(addr) {
                Some(sender) => sender,
                None => continue,
            };

            if let Err(e) = sender.unbounded_send(Message::Text(message.clone().into())) {
                println!("Failed to send message to {addr}: {e}");
            }
        }
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
