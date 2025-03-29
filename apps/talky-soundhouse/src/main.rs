use anyhow::anyhow;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde_json::{json, Value};
use std::any::TypeId;
use std::convert::Infallible;
use std::fmt::Binary;
use std::sync::Arc;
use std::{any::Any, collections::HashMap};
use tokio::sync::Mutex;
use uuid::Uuid;
use warp::{
    filters::ws::{Message, WebSocket},
    Filter, Reply,
};

#[derive(Clone)]
struct ClientInfo {
    id: String,
    role: String,
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
}

#[derive(Clone)]
struct Room {
    clients: Vec<ClientInfo>,
}

type Rooms = Arc<Mutex<HashMap<String, Room>>>;

#[tokio::main]
async fn main() {
    println!("Starting soundhouse server...");

    let rooms: Rooms = Arc::new(Mutex::new(HashMap::new()));

    let soundhouse = warp::path("soundhouse")
        .and(warp::ws())
        .and(with_rooms(rooms.clone()))
        .map(|ws: warp::ws::Ws, rooms: Rooms| {
            ws.on_upgrade(move |socket| handle_connection(socket, rooms))
        });

    println!("Server running at ws://0.0.0.0:8080/soundhouse");
    warp::serve(soundhouse).run(([0, 0, 0, 0], 8080)).await;
}

fn with_rooms(rooms: Rooms) -> impl Filter<Extract = (Rooms,), Error = Infallible> + Clone {
    warp::any().map(move || rooms.clone())
}

async fn handle_connection(ws: WebSocket, rooms: Rooms) {
    println!("New connection established");
    let (sender, mut receiver) = ws.split();
    let sender = Arc::new(Mutex::new(sender));

    let init_msg = match receiver.next().await {
        Some(Ok(msg)) if msg.is_text() => msg.to_str().unwrap().to_string(),
        _ => {
            println!("No valid init message received");
            return;
        }
    };

    let init_json: Value = match serde_json::from_str(&init_msg) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to parse init message: {:?}", e);
            return;
        }
    };

    let join_code = get_str_or_error(&init_json, "join_code").unwrap_or_default();
    let auth_code = get_str_or_error(&init_json, "auth_code").unwrap_or_default();
    let role = get_str_or_error(&init_json, "role").unwrap_or_default();

    let user = match talky_auth::JwtService::decode(auth_code) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid user token: {:?}", auth_code);
            return;
        }
    };

    let client_id = Uuid::new_v4().to_string();

    println!("Client {:?} joined room '{}' as {}", user, join_code, role);

    let client_info = ClientInfo {
        id: client_id.clone(),
        role: role.to_string(),
        sender: sender.clone(),
    };

    {
        let mut rooms_lock = rooms.lock().await;
        let room = rooms_lock.entry(join_code.to_string()).or_insert(Room {
            clients: Vec::new(),
        });
        room.clients.push(client_info.clone());

        broadcast_active_clients(join_code, room).await;
    }

    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) if msg.is_text() => {
                let text = msg.to_str().unwrap().to_string();
                println!("Received message from {}: {}", client_id, text);

                let rooms_lock = rooms.lock().await;
                if let Some(room) = rooms_lock.get(join_code) {
                    for client in &room.clients {
                        if client.id != client_id {
                            let mut s = client.sender.lock().await;
                            if let Err(e) = s.send(Message::text(text.clone())).await {
                                eprintln!("Error sending message to client {}: {:?}", client.id, e);
                            }
                        }
                    }
                }
            }

            Ok(_) => {
                eprintln!("Received non-text message");
            }
            Err(e) => {
                eprintln!("Error receiving message: {:?}", e);
                break;
            }
        }
    }

    {
        let mut rooms_lock = rooms.lock().await;
        if let Some(room) = rooms_lock.get_mut(join_code) {
            room.clients.retain(|c| c.id != client_id);
            broadcast_active_clients(join_code, room).await;
            if room.clients.is_empty() {
                rooms_lock.remove(join_code);
            }
        }
    }
    println!("Client {} disconnected", client_id);
}

async fn broadcast_active_clients(join_code: &str, room: &Room) {
    let active_clients: Vec<_> = room
        .clients
        .iter()
        .map(|c| {
            json!({
                "id": c.id,
                "role": c.role,
            })
        })
        .collect();

    let update_msg = json!({
        "type": "active_clients",
        "join_code": join_code,
        "clients": active_clients,
    })
    .to_string();

    for client in &room.clients {
        let mut s = client.sender.lock().await;
        if let Err(e) = s.send(Message::text(update_msg.clone())).await {
            eprintln!(
                "Error sending active clients update to {}: {:?}",
                client.id, e
            );
        }
    }
}

fn get_str_or_error<'a>(json: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    json.get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("Missing or empty field: {}", key))
}
