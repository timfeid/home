use chrono::Utc;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use std::{collections::HashMap, sync::Arc};
use talky_auth::Claims;
use tokio::sync::Mutex;
use ulid::Ulid;
use warp::filters::ws::{Message, WebSocket};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct ClientInfo {
    id: String,
    user: Claims,
    role: String,
    last_pong: u32,
    last_ping: u32,
    #[serde(skip_serializing, skip_deserializing)]
    sender: Option<Arc<Mutex<SplitSink<WebSocket, Message>>>>,
}

#[derive(Deserialize, Serialize)]
pub struct Lobby {
    pub presence: HashMap<String, ClientInfo>,
    pub join_code: String,
}

impl Lobby {
    pub fn new(join_code: String) -> Self {
        Lobby {
            presence: HashMap::new(),
            join_code,
        }
    }

    pub fn remove_connection(&mut self, client_id: &String) {
        self.presence.remove(client_id);
    }

    pub fn add_connection(
        &mut self,
        sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
        user: Claims,
        role: String,
    ) {
        let client_id = Ulid::new().to_string();

        println!(
            "Client {:?} joined room '{}' as {}",
            user, self.join_code, role
        );

        let time = Utc::now().timestamp() as u32;
        let client_info = ClientInfo {
            id: client_id.clone(),
            user,
            role,
            sender: Some(sender.clone()),
            last_ping: time.clone(),
            last_pong: time,
        };
        self.presence.insert(client_id, client_info);

        // {
        //     let mut rooms_lock = rooms.lock().await;
        //     let room = rooms_lock.entry(join_code.to_string()).or_insert(Room {
        //         clients: Vec::new(),
        //     });
        //     room.clients.push(client_info.clone());

        //     broadcast_active_clients(join_code, room).await;
        // }
    }
}
