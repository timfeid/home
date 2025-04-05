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

impl ClientInfo {
    pub async fn send(&self, message: &OutgoingMessage) -> AppResult<()> {
        self.sender
            .lock()
            .await
            .send(
                message
                    .to_ws_message()
                    .map_err(|e| AppError::Anyhow(anyhow::Error::msg(e)))?,
            )
            .await?;

        Ok(())
    }
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
                user_id: c.user_id.to_string(),
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

    pub async fn add_client(&self, client_info: ClientInfo) -> AppResult<()> {
        self.clients
            .lock()
            .await
            .insert(client_info.id.clone(), client_info.clone());
        self.broadcast_active_clients().await;

        Ok(())
    }

    pub async fn remove_client(&self, client_id: &str) {
        self.clients.lock().await.remove(client_id);
        self.broadcast_active_clients().await;
    }

    pub async fn send_to_client(
        &self,
        client_id: &str,
        message: &OutgoingMessage,
    ) -> AppResult<()> {
        match self.clients.lock().await.get(client_id) {
            Some(client) => client.send(message).await,
            _ => Err(AppError::Anyhow(anyhow::anyhow!("Client not found."))),
        }
    }

    pub async fn handle_webrtc_signal(
        &self,
        sender_client_id: &str,
        target_client_id: &str,
        signal_data: serde_json::Value,
    ) -> AppResult<()> {
        let rooms_guard = self.rooms_lock().await;
        let message = OutgoingMessage::WebRtcSignal {
            sender_client_id: (*sender_client_id).to_string(),
            signal_data,
        };

        if let Err(e) = self.send_to_client(target_client_id, &message).await {
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
        Ok(())
    }

    pub async fn handle_chat_message(
        &self,
        sender_id: &str,
        channel_id: String,
        content: String,
    ) -> AppResult<()> {
        let user_id = {
            self.clients
                .lock()
                .await
                .get(sender_id)
                .and_then(|s| Some(s.user_id.clone()))
        };
        if let Some(user_id) = user_id {
            let message = OutgoingMessage::ChatMessageBroadcast {
                sender_id: (*sender_id).to_string(),
                user_id,
                channel_id,
                content,
            };

            self.broadcast_all(&message).await;
        }
        Ok(())
    }

    async fn get_client_info_msgs(&self) -> Vec<ClientInfoMsg> {
        self.clients
            .lock()
            .await
            .values()
            .map(|c| ClientInfoMsg {
                user_id: c.user_id.to_string(),
            })
            .collect()
    }

    async fn broadcast_all(&self, message: &OutgoingMessage) {
        let ws_message = match message.to_ws_message() {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        let clients = { self.clients.lock().await.clone() };
        for (id, client) in clients.iter() {
            let mut sender_lock = client.sender.lock().await;
            if let Err(e) = sender_lock.send(ws_message.clone()).await {
                tracing::warn!("Failed to send broadcast message to client {}: {}", id, e);
            }
        }
    }

    async fn broadcast_active_clients(&self) {
        let client_infos = self.get_client_info_msgs().await;
        let update_msg = OutgoingMessage::ActiveClientsUpdate {
            clients: client_infos,
        };

        self.broadcast_all(&update_msg).await;
        tracing::debug!("Broadcasted active clients");
    }

    // async fn broadcast_active_clients(&self, join_code: &str, room: &Room) {
    //     let client_infos = room.get_client_info_msgs();
    //     let update_msg = OutgoingMessage::ActiveClientsUpdate {
    //         join_code: join_code.to_string(),
    //         clients: client_infos,
    //     };

    //     room.broadcast_all(&update_msg).await;
    //     tracing::debug!("Broadcasted active clients for room '{}'", join_code);
    // }
}
