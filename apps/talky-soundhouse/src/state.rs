use crate::error::{AppError, AppResult};
use crate::message::{ClientInfoMsg, OutgoingMessage};
use futures::{stream::SplitSink, SinkExt};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use warp::ws::{Message, WebSocket};

pub type ClientSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

#[derive(Clone)]
pub struct ClientInfo {
    pub id: String,
    pub user_id: String,
    pub role: String,
    pub sender: ClientSender,
}

#[derive(Default)]
pub struct Room {
    pub clients: HashMap<String, ClientInfo>,
}

impl Room {
    fn add_client(&mut self, client: ClientInfo) -> bool {
        self.clients.insert(client.id.clone(), client).is_none()
    }

    fn remove_client(&mut self, client_id: &str) -> Option<ClientInfo> {
        self.clients.remove(client_id)
    }

    fn get_client_info_msgs(&self) -> Vec<ClientInfoMsg> {
        self.clients
            .values()
            .map(|c| ClientInfoMsg {
                id: c.id.to_string(),
                role: c.role.clone(),
            })
            .collect()
    }

    async fn send_to_client(
        &self,
        target_client_id: &str,
        message: &OutgoingMessage,
    ) -> AppResult<()> {
        if let Some(client) = self.clients.get(target_client_id) {
            let ws_message = message
                .to_ws_message()
                .map_err(|e| AppError::Anyhow(anyhow::Error::msg(e)))?;
            client
                .sender
                .lock()
                .await
                .send(ws_message)
                .await
                .map_err(|e| {
                    tracing::error!(
                        "Failed to send message to client {}: {}",
                        target_client_id,
                        e
                    );
                    AppError::ClientSendError
                })?;
            Ok(())
        } else {
            tracing::warn!(
                "Attempted to send message to non-existent client {} in room",
                target_client_id
            );

            Ok(())
        }
    }

    async fn broadcast_except(&self, sender_id: &str, message: &OutgoingMessage) {
        let ws_message = match message.to_ws_message() {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        for (id, client) in self.clients.iter() {
            if id != sender_id {
                let mut sender_lock = client.sender.lock().await;
                if let Err(e) = sender_lock.send(ws_message.clone()).await {
                    tracing::warn!("Failed to send broadcast message to client {}: {}", id, e);
                }
            }
        }
    }

    async fn broadcast_all(&self, message: &OutgoingMessage) {
        let ws_message = match message.to_ws_message() {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        for (id, client) in self.clients.iter() {
            let mut sender_lock = client.sender.lock().await;
            if let Err(e) = sender_lock.send(ws_message.clone()).await {
                tracing::warn!("Failed to send broadcast message to client {}: {}", id, e);
            }
        }
    }
}

type RoomsMap = HashMap<String, Room>;

#[derive(Clone)]
pub struct AppState {
    pub rooms: Arc<Mutex<RoomsMap>>,
    pub clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            rooms: Arc::new(Mutex::new(HashMap::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn rooms_lock(&self) -> MutexGuard<'_, RoomsMap> {
        self.rooms.lock().await
    }

    pub async fn add_client_to_room(
        &self,
        join_code: String,
        client_info: ClientInfo,
    ) -> AppResult<()> {
        let mut rooms_guard = self.rooms_lock().await;
        let room = rooms_guard.entry(join_code.clone()).or_default();

        let client_id = client_info.id.clone();
        if !room.add_client(client_info) {
            tracing::warn!(
                "Client ID {} already exists in room {}",
                client_id,
                join_code
            );
        }

        tracing::info!("Client {} added to room '{}'", client_id, join_code);

        self.broadcast_active_clients(&join_code, room).await;

        Ok(())
    }

    pub async fn handle_webrtc_signal(
        &self,
        join_code: &str,
        sender_client_id: &str,
        target_client_id: &str,
        signal_data: serde_json::Value,
    ) -> AppResult<()> {
        let rooms_guard = self.rooms_lock().await;
        if let Some(room) = rooms_guard.get(join_code) {
            let message = OutgoingMessage::WebRtcSignal {
                sender_client_id: (*sender_client_id).to_string(),
                signal_data,
            };

            if let Err(e) = room.send_to_client(target_client_id, &message).await {
                tracing::error!(
                    "Failed to relay WebRTC signal from {} to {}: {:?}",
                    sender_client_id,
                    target_client_id,
                    e
                );

                return Err(e);
            } else {
                tracing::debug!(
                    "Relayed WebRTC signal from {} to {}",
                    sender_client_id,
                    target_client_id
                );
            }
        } else {
            tracing::warn!(
                "Cannot relay WebRTC signal: Room '{}' not found for sender {}",
                join_code,
                sender_client_id
            );
        }
        Ok(())
    }

    pub async fn handle_chat_message(
        &self,
        join_code: &str,
        sender_id: &str,
        sender_role: &str,
        content: String,
    ) {
        let rooms_guard = self.rooms_lock().await;
        if let Some(room) = rooms_guard.get(join_code) {
            let message = OutgoingMessage::ChatMessageBroadcast {
                sender_id: (*sender_id).to_string(),
                sender_role: sender_role.to_string(),
                content,
            };
            room.broadcast_except(sender_id, &message).await;
        } else {
            tracing::warn!(
                "Cannot broadcast chat message: Room '{}' not found for sender {}",
                join_code,
                sender_id
            );
        }
    }

    pub async fn remove_client_from_room(&self, join_code: &str, client_id: &str) {
        let mut rooms_guard = self.rooms_lock().await;
        let mut room_removed = false;

        if let Some(room) = rooms_guard.get_mut(join_code) {
            if room.remove_client(client_id).is_some() {
                tracing::info!("Client {} removed from room '{}'", client_id, join_code);

                if room.clients.is_empty() {
                    room_removed = true;
                    tracing::info!("Room '{}' is empty, removing.", join_code);
                } else {
                    self.broadcast_active_clients(join_code, room).await;
                }
            } else {
                tracing::warn!(
                    "Attempted to remove non-existent client {} from room {}",
                    client_id,
                    join_code
                );
            }
        } else {
            tracing::warn!(
                "Attempted to remove client {} from non-existent room {}",
                client_id,
                join_code
            );
        }

        if room_removed {
            rooms_guard.remove(join_code);
        }
    }

    async fn broadcast_active_clients(&self, join_code: &str, room: &Room) {
        let client_infos = room.get_client_info_msgs();
        let update_msg = OutgoingMessage::ActiveClientsUpdate {
            join_code: join_code.to_string(),
            clients: client_infos,
        };

        room.broadcast_all(&update_msg).await;
        tracing::debug!("Broadcasted active clients for room '{}'", join_code);
    }
}
