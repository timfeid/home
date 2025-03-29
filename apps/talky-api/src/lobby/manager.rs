use chrono::Utc;
use futures::stream::StreamExt;
use sqlx::types::time::{Date, PrimitiveDateTime};
use talky_auth::Claims;
use tokio_stream::wrappers::BroadcastStream;

use futures::{future, Stream};
use jsonwebtoken::TokenData;
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use tokio::sync::{broadcast, mpsc};
use tokio::task;
use tokio::time::{interval, timeout};
use tokio_stream::wrappers::ReceiverStream;
use ulid::Ulid;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use super::lobby::{self, Lobby, LobbyCommand, LobbyData, PresenceMember};
use crate::error::{AppError, AppResult};
use crate::lobby::lobby::LobbyCommandWrapper;

#[derive(Clone)]
pub struct LobbyManager {
    lobbies: Arc<Mutex<HashMap<String, Arc<Mutex<Lobby>>>>>,
}

#[derive(Type, Deserialize, Clone, Serialize, Debug)]
pub struct LobbyTurnMessage {
    pub messages: Vec<String>,
}

#[derive(Type, Deserialize, Clone, Serialize)]
pub struct ModalButton {
    pub id: String,
    pub text: String,

    #[serde(skip_serializing, skip_deserializing)]
    pub action: Option<
        Arc<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>,
    >,
}
impl ModalButton {
    pub(crate) fn new<F>(text: &str, action: F) -> Self
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync + 'static,
    {
        Self {
            id: Ulid::new().to_string(),
            text: text.to_string(),
            action: Some(Arc::new(action)),
        }
    }
}

impl std::fmt::Debug for ModalButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalButton")
            .field("id", &self.id)
            .field("text", &self.text)
            .finish()
    }
}

impl std::fmt::Debug for LobbyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LobbyManager")
            .field("lobbies", &self.lobbies)
            .finish()
    }
}

impl LobbyManager {
    pub async fn create_lobby(self: &Arc<Self>, user: &Claims) -> AppResult<String> {
        let mut lobbies = self.lobbies.lock().await;
        let lobby = Lobby::new(user).await;
        let lobby_id = Ulid::new().to_string();
        let join_code = lobby_id.clone();

        lobbies.insert(lobby_id.clone(), Arc::new(Mutex::new(lobby)));

        let lobby_clone = Arc::clone(self);

        tokio::spawn(async move {
            let tick_duration = Duration::from_millis(150);
            let mut ticker = interval(tick_duration);
            loop {
                ticker.tick().await;
                {
                    if let Err(some) = lobby_clone.get_lobby(&join_code).await {
                        println!("Error getting lobby? {:?}", some);
                        break;
                    }
                    if let Err(e) = lobby_clone.tick_lobby(&join_code).await {
                        println!("Error within tick {:?}", e);
                    }
                }
            }
        });

        Ok(lobby_id)
    }

    pub async fn get_lobby(&self, join_code: &String) -> AppResult<Arc<Mutex<Lobby>>> {
        let lobbies = self.lobbies.lock().await;

        let lobby = lobbies
            .get(join_code)
            .ok_or(AppError::BadRequest("Lobby not found".to_owned()))?
            .clone();

        Ok(lobby)
    }

    pub async fn pong(
        &self,
        join_code: &String,
        socket_id: &String,
        claims: Claims,
    ) -> AppResult<()> {
        let lobby = self.get_lobby(&join_code).await?;
        if let Some(presence) = lobby.lock().await.data.presence.get_mut(socket_id) {
            presence.last_pong = Utc::now().timestamp() as u32;
        }

        Ok(())
    }

    pub async fn subscribe_to_lobby_updates(
        &self,
        lobby_id: String,
        claims: Claims,
    ) -> AppResult<impl tokio_stream::Stream<Item = LobbyCommand>> {
        let lobby_arc = self.get_lobby(&lobby_id).await?;

        let pub_tx = {
            let lobby = lobby_arc.lock().await;
            lobby.pub_tx.clone().ok_or(AppError::InternalServerError(
                "PubSub not initialized".to_owned(),
            ))?
        };

        let rx = pub_tx.subscribe();
        let socket_id = Ulid::new().to_string();
        let socket_id_clone = socket_id.clone();

        let user_id = claims.clone().sub.to_string();
        let stream = BroadcastStream::new(rx).filter_map(move |result| {
            let claims_cl = claims.clone();
            let socket_id = socket_id_clone.clone();
            async move {
                match result {
                    Ok(data) => match data.audience {
                        super::lobby::LobbyCommandAudience::All => Some(data.command),
                        super::lobby::LobbyCommandAudience::Socket(sock) => {
                            if sock == socket_id {
                                Some(data.command)
                            } else {
                                None
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Error receiving broadcast: {:?}", e);
                        None
                    }
                }
            }
        });

        {
            let mut lobby = lobby_arc.lock().await;
            lobby.data.presence.insert(
                socket_id,
                PresenceMember {
                    user_id,
                    last_pong: Utc::now().timestamp() as u32,
                    last_ping: Utc::now().timestamp() as u32,
                },
            )
        };

        self.notify_presence_updated(&lobby_id).await.ok();

        Ok(stream)
    }

    pub async fn tick_lobby(&self, lobby_id: &String) -> AppResult<()> {
        let mut notify = false;
        let now = Utc::now().timestamp() as u32;

        let lobby_arc = self.get_lobby(lobby_id).await?;

        let mut remove_sockets: Vec<String> = vec![];
        let mut send_pings: Vec<String> = vec![];
        {
            let mut lobby = lobby_arc.lock().await;
            for (socket_id, presence) in lobby.data.presence.iter_mut() {
                let since_last_pong = now as i64 - presence.last_pong as i64;
                let since_last_ping = now as i64 - presence.last_ping as i64;
                if since_last_pong > 60 {
                    println!("No pong in over a minute, oh no");
                    remove_sockets.push(socket_id.clone());
                    continue;
                }
                if since_last_ping == 20 {
                    println!("Haven't seen {socket_id} in a while...sending ping");
                    presence.last_ping = now;
                    send_pings.push(socket_id.clone());
                    println!("updated last ping.");
                }
            }
        }

        {
            let mut lobby = lobby_arc.lock().await;
            if remove_sockets.len() > 0 {
                notify = true;
            }
            for socket_id in remove_sockets {
                lobby.data.presence.remove(&socket_id);
            }
        }

        {
            for socket_id in &send_pings {
                self.send_individual(lobby_id, socket_id, LobbyCommand::Ping(socket_id.clone()))
                    .await?;
            }
        }

        if notify {
            self.notify_presence_updated(lobby_id).await?;
        }
        Ok(())
    }

    pub async fn send_individual(
        &self,
        lobby_id: &String,
        socket_id: &String,
        data: LobbyCommand,
    ) -> AppResult<()> {
        let lobby_arc = self.get_lobby(lobby_id).await?;

        let pub_tx = {
            let lobby = lobby_arc.lock().await;
            lobby.pub_tx.clone().ok_or(AppError::InternalServerError(
                "PubSub not initialized".to_string(),
            ))?
        };

        pub_tx
            .send(LobbyCommandWrapper::new_individual(data, socket_id.clone()))
            .map_err(|x| AppError::InternalServerError("Something".to_string()))?;
        Ok(())
    }

    pub async fn notify_presence_updated(&self, lobby_id: &str) -> AppResult<()> {
        println!("Notifying lobby of socket updates");
        let lobby_arc = self.get_lobby(&lobby_id.to_string()).await?;

        let (lobby_data, pub_tx) = {
            let lobby = lobby_arc.lock().await;
            (
                lobby.data.clone(),
                lobby.pub_tx.clone().ok_or(AppError::InternalServerError(
                    "PubSub not initialized".to_string(),
                ))?,
            )
        };

        pub_tx
            .send(LobbyCommandWrapper::new_all(LobbyCommand::Data(lobby_data)))
            .map_err(|x| AppError::InternalServerError("Something".to_string()))?;
        Ok(())
    }

    pub async fn join_lobby(&self, lobby_id: &str, user: &Claims) -> Option<()> {
        {
            let hash_map = self.lobbies.lock().await;
            let lobby = hash_map.get(lobby_id)?;
            lobby.lock().await.join(user).await;
        }

        self.notify_presence_updated(lobby_id).await.ok();

        Some(())
    }

    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
