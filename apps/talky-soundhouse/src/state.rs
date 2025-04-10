use crate::error::{AppError, AppResult};
use crate::message::{ClientInfoMsg, OutgoingMessage};
use futures::{stream::SplitSink, SinkExt};
use serde::Serialize;
use serde_json::Value;
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use talky_data::database::create_connection;
use talky_services::channel::service::ChannelService;
use talky_services::lobby::service::{LobbyResource, LobbyService};
use talky_services::message::service::{AddChatMessageArgs, MessageResource, MessageService};
use talky_services::niche::service::NicheService;
use talky_services::DatabasePool;
use tokio::sync::{Mutex, MutexGuard};
use warp::ws::{Message, WebSocket};

pub type ClientSender = Arc<Mutex<SplitSink<WebSocket, Message>>>;

#[derive(Clone, Debug)]
pub struct RoomClientInfo {
    pub client: ClientInfo,
    pub role: String,
}

impl RoomClientInfo {
    pub fn get_resource(&self) -> UserRoomResource {
        UserRoomResource {
            user: self.client.resource.clone(),
            role: self.role.clone(),
        }
    }
}

#[derive(Type, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct UserRoomResource {
    pub user: UserResource,
    pub role: String,
}

#[derive(Type, Serialize, Debug, Clone, Hash, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct UserResource {
    pub user_id: String,
}

#[derive(Clone, Debug)]
pub struct ClientInfo {
    pub id: String,
    pub sender: ClientSender,
    pub current_niche_id: Option<String>,
    pub resource: UserResource,
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

type UserId = String;
type NicheId = String;
type ClientId = String;
#[derive(Type, Serialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct RoomResource {
    users: HashMap<UserId, Vec<UserRoomResource>>,
}
impl RoomResource {
    async fn generate_active_lobbies_message(rooms: &HashMap<NicheId, Room>) -> OutgoingMessage {
        let mut channels = HashMap::new();
        for (channel_id, room) in rooms.iter() {
            channels.insert(channel_id.clone(), room.to_resource().await);
        }

        OutgoingMessage::ActiveChannels { channels }
    }
}

#[derive(Debug)]
pub struct Room {
    clients: Arc<Mutex<HashMap<ClientId, RoomClientInfo>>>,
    channel: LobbyResource,
}

impl Room {
    fn new(channel: LobbyResource) -> Self {
        let clients = Arc::new(Mutex::new(HashMap::new()));

        Self { clients, channel }
    }

    async fn add_client(&self, client: ClientInfo, role: String) {
        let client_id = client.id.clone();
        let room_client = RoomClientInfo { role, client };
        self.clients.lock().await.insert(client_id, room_client);
    }

    pub fn get_channel(&self) -> &LobbyResource {
        &self.channel
    }

    pub async fn to_resource(&self) -> RoomResource {
        let mut users: HashMap<String, Vec<UserRoomResource>> = HashMap::new();
        for client_info in self.clients.lock().await.values() {
            let user_id = client_info.get_resource().user.user_id.clone();
            users
                .entry(user_id)
                .or_insert_with(Vec::new)
                .push(client_info.get_resource());
        }

        RoomResource { users }
    }

    pub async fn remove_client(&self, client_id: &str) {
        self.clients.lock().await.remove(client_id);
    }
}

#[derive(Clone)]
pub struct AppState {
    clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
    // niche_id -> channel_id -> room
    lobbies: Arc<Mutex<HashMap<String, HashMap<String, Room>>>>,
    connection: DatabasePool,
}

impl AppState {
    pub async fn new(database_url: &String) -> Self {
        AppState {
            clients: Arc::new(Mutex::new(HashMap::new())),
            connection: create_connection(database_url).await,
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        }
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
        let niche_id = {
            let mut clients_locked = self.clients.lock().await;
            if let Some(client) = clients_locked.remove(client_id) {
                client.current_niche_id.clone()
            } else {
                None
            }
        };

        self.remove_client_from_current_room(client_id).await;

        self.broadcast_active_clients().await;
    }

    pub async fn remove_client_from_current_room(&self, client_id: &str) {
        let niche_keys: Vec<String> = {
            let rooms_locked = self.lobbies.lock().await;
            rooms_locked.keys().cloned().collect()
        };

        let mut niches_to_notify = Vec::new();

        for niche_key in niche_keys.iter() {
            let lock = self.lobbies.lock().await;
            let lobby_map = lock.get(niche_key);

            if let Some(lobby_map) = lobby_map {
                let client_removed = {
                    let mut client_removed = false;
                    for room in lobby_map.values() {
                        let mut clients_locked = room.clients.lock().await;
                        if clients_locked.remove(client_id).is_some() {
                            client_removed = true;
                        }
                    }
                    client_removed
                };

                if client_removed {
                    niches_to_notify.push(niche_key.clone());
                }
            }
        }

        for niche_key in niches_to_notify.iter() {
            self.broadcast_niche_clients(niche_key).await;
            // we should really clean up empty "rooms"
        }
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

    pub async fn update_niche(&self, client_id: &str, niche_id: &str) -> AppResult<()> {
        self.clients
            .lock()
            .await
            .get_mut(client_id)
            .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("Client not found")))?
            .current_niche_id = Some(niche_id.to_string());

        if let Some(lobbies) = self.lobbies.lock().await.get(niche_id) {
            let message = RoomResource::generate_active_lobbies_message(lobbies).await;
            self.send_to_client(client_id, &message).await?;
        }

        Ok(())
    }

    pub async fn join(&self, client_id: &str, lobby_id: String, role: String) -> AppResult<()> {
        tracing::info!("Client {} is attempting to join {}...", client_id, lobby_id);
        let lobby_service = LobbyService::new(self.connection.clone());
        let lobby = lobby_service
            .find_by_id(lobby_id.clone())
            .await
            .map_err(AppError::from)?;

        let client = self
            .clients
            .lock()
            .await
            .get(client_id)
            .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("Client not found")))?
            .clone();

        self.remove_client_from_current_room(client_id).await;

        let lobby_niche_id = lobby.niche_id.clone();
        let lobby_id = lobby.id.clone();

        self.update_niche(client_id, &lobby_niche_id).await?;

        {
            let mut lobbies = self.lobbies.lock().await;
            let rooms = lobbies.entry(lobby_niche_id.clone()).or_default();

            let room = rooms.entry(lobby_id.clone()).or_insert(Room::new(lobby));

            room.add_client(client, role).await;
        }

        tracing::info!(
            "Added client with id {} to room with id {}. ",
            client_id,
            lobby_id
        );

        self.broadcast_niche_clients(&lobby_niche_id).await;

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
                .and_then(|s| Some(s.resource.user_id.clone()))
        };

        if let Some(user_id) = user_id {
            let message_service = MessageService::new(self.connection.clone());
            let message = message_service
                .add_chat_message(AddChatMessageArgs {
                    user_id,
                    channel_id: channel_id.clone(),
                    contents: content,
                })
                .await;

            let broadcast_message = OutgoingMessage::ChatMessageBroadcast {
                sender_id: (*sender_id).to_string(),
                channel_id: channel_id.clone(),
                message,
            };

            self.broadcast_all(&broadcast_message).await;
        }
        Ok(())
    }

    async fn get_client_info_msgs(&self) -> Vec<ClientInfoMsg> {
        self.clients
            .lock()
            .await
            .values()
            .map(|c| ClientInfoMsg {
                user_id: c.resource.user_id.to_string(),
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

    async fn broadcast_niche<F>(&self, filter: F, message: &OutgoingMessage)
    where
        F: Fn(&ClientInfo) -> bool + Send + Sync,
    {
        let ws_message = match message.to_ws_message() {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!("Failed to serialize broadcast message: {}", e);
                return;
            }
        };

        // Collect relevant clients without holding the lock
        let clients: Vec<(ClientSender, String)> = {
            self.clients
                .lock()
                .await
                .values()
                .filter(|&client| filter(client))
                .map(|client| (client.sender.clone(), client.id.clone()))
                .collect()
        };

        for (sender, client_id) in clients.iter() {
            let mut sender_lock = sender.lock().await;
            if let Err(e) = sender_lock.send(ws_message.clone()).await {
                tracing::warn!("Failed to send broadcast message: {}", e);
            } else {
                tracing::info!("Sent message {:?} to client {}.", message, client_id);
            }
        }
    }

    async fn broadcast_active_clients(&self) {
        let client_infos = self.get_client_info_msgs().await;
        let update_msg = OutgoingMessage::ActiveClientsUpdate {
            clients: client_infos,
        };

        // todo broadcast to all of client's niches
        // self.broadcast_niches(niche_ids, &update_msg).await;
        self.broadcast_all(&update_msg).await;
        tracing::info!("Broadcasted active clients");
    }

    pub async fn broadcast_niche_clients_except(
        &self,
        sender_id: &str,
        niche_id: &str,
        message: &OutgoingMessage,
    ) {
        tracing::info!("Broadcasting to niche with id {}...", niche_id);

        // Retrieve and drop the lock before generating the message
        {
            if let Some(lobbies) = self.lobbies.lock().await.get(niche_id) {
                self.broadcast_niche(
                    |client| {
                        client.current_niche_id.as_ref().map(|s| s.as_str()) == Some(niche_id)
                            && client.id != sender_id
                    },
                    &message,
                )
                .await;
            }
        };

        tracing::info!("Done!");
    }

    async fn broadcast_niche_clients(&self, niche_id: &str) {
        tracing::info!("Broadcasting to niche with id {}...", niche_id);

        // Retrieve and drop the lock before generating the message
        {
            if let Some(lobbies) = self.lobbies.lock().await.get(niche_id) {
                let message = RoomResource::generate_active_lobbies_message(lobbies).await;
                self.broadcast_niche(
                    |client| client.current_niche_id.as_ref().map(|s| s.as_str()) == Some(niche_id),
                    &message,
                )
                .await;
            }
        };

        tracing::info!("Done!");
    }

    pub(crate) async fn answer(
        &self,
        client_id: &str,
        sdp: String,
        channel_id: String,
        niche_id: String,
    ) -> AppResult<()> {
        let valid_client_ids: HashSet<String> = {
            let rooms_locked = self.lobbies.lock().await;
            match rooms_locked.get(&niche_id) {
                Some(channels) => {
                    if let Some(room) = channels.get(&channel_id) {
                        room.clients.lock().await.keys().cloned().collect()
                    } else {
                        HashSet::new()
                    }
                }
                None => HashSet::new(),
            }
        };
        self.broadcast_niche(
            |client| client_id != client.id && valid_client_ids.contains(&client.id),
            &OutgoingMessage::Answer { answer: sdp },
        )
        .await;

        Ok(())
    }

    pub(crate) async fn candidate(
        &self,
        client_id: &str,
        candidate: Value,
        channel_id: String,
        niche_id: String,
    ) -> AppResult<()> {
        let valid_client_ids: HashSet<String> = {
            let rooms_locked = self.lobbies.lock().await;
            match rooms_locked.get(&niche_id) {
                Some(channels) => {
                    if let Some(room) = channels.get(&channel_id) {
                        room.clients.lock().await.keys().cloned().collect()
                    } else {
                        HashSet::new()
                    }
                }
                None => HashSet::new(),
            }
        };

        self.broadcast_niche(
            |client| client_id != client.id && valid_client_ids.contains(&client.id),
            &OutgoingMessage::Candidate { candidate },
        )
        .await;

        Ok(())
    }

    pub(crate) async fn offer(
        &self,
        client_id: &str,
        offer: String,
        channel_id: String,
        niche_id: String,
    ) -> AppResult<()> {
        let valid_client_ids: HashSet<String> = {
            let rooms_locked = self.lobbies.lock().await;
            match rooms_locked.get(&niche_id) {
                Some(channels) => {
                    if let Some(room) = channels.get(&channel_id) {
                        room.clients.lock().await.keys().cloned().collect()
                    } else {
                        HashSet::new()
                    }
                }
                None => HashSet::new(),
            }
        };

        self.broadcast_niche(
            |client| client_id != client.id && valid_client_ids.contains(&client.id),
            &OutgoingMessage::Offer { offer },
        )
        .await;

        Ok(())
    }
}
