use chrono::Utc;
use futures::stream::StreamExt;
use sqlx::types::time::{Date, PrimitiveDateTime};
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

use super::lobby::{Lobby, LobbyData, PresenceMember};
use crate::error::{AppError, AppResult};
use crate::services::jwt::{Claims, JwtService};

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
                    if let Ok(some) = lobby_clone.get_lobby(&join_code).await {
                        lobby_clone.tick_lobby(&join_code).await.ok();
                    } else {
                        println!("Lobby no longer running?");
                        break;
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

    pub async fn subscribe_to_lobby_updates(
        &self,
        lobby_id: String,
        claims: Claims,
    ) -> AppResult<impl tokio_stream::Stream<Item = LobbyData>> {
        let lobby_arc = {
            let lobbies = self.lobbies.lock().await;
            lobbies
                .get(&lobby_id)
                .ok_or(AppError::BadRequest("Lobby not found".to_owned()))?
                .clone()
        };

        let pub_tx = {
            let lobby = lobby_arc.lock().await;
            lobby.pub_tx.clone().ok_or(AppError::InternalServerError(
                "PubSub not initialized".to_owned(),
            ))?
        };

        let rx = pub_tx.subscribe();

        let user_id = claims.clone().sub.to_string();
        let stream = BroadcastStream::new(rx).filter_map(move |result| {
            let claims_cl = claims.clone();
            async move {
                match result {
                    Ok(data) => Some(data),
                    Err(e) => {
                        eprintln!("Error receiving broadcast: {:?}", e);
                        None
                    }
                }
            }
        });

        let socket_id = Ulid::new().to_string();
        {
            let mut lobby = lobby_arc.lock().await;
            lobby.data.presence.insert(
                socket_id,
                PresenceMember {
                    user_id,
                    last_seen: Utc::now().timestamp() as u32,
                },
            )
        };

        self.notify_lobby(&lobby_id).await.ok();

        Ok(stream)
    }

    pub async fn tick_lobby(&self, lobby_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut notify = false;

        let lobby_arc = {
            let lobbies = self.lobbies.lock().await;
            lobbies.get(lobby_id).ok_or("Lobby not found")?.clone()
        };

        let mut remove_sockets: Vec<String> = vec![];
        {
            let lobby = lobby_arc.lock().await.data.clone();
            for (socket_id, presence) in lobby.presence.iter() {
                let now = Utc::now().timestamp() as u32;
                let diff = now as i64 - presence.last_seen as i64;
                if diff > 20 {
                    println!("Haven't seen {socket_id} in a while...sending ping");
                    println!("jk for now we just remove them CYA");
                    remove_sockets.push(socket_id.clone());
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

        if notify {
            self.notify_lobby(lobby_id).await.ok();
        }
        // let (lobby_data, pub_tx) = {
        //     let lobby = lobby_arc.lock().await;
        //     (
        //         lobby.data.clone(),
        //         lobby.pub_tx.clone().ok_or("PubSub not initialized")?,
        //     )
        // };

        // pub_tx.send(lobby_data)?;
        Ok(())
    }

    pub async fn notify_lobby(&self, lobby_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("Notifying lobby of socket updates");
        let lobby_arc = {
            let lobbies = self.lobbies.lock().await;
            lobbies.get(lobby_id).ok_or("Lobby not found")?.clone()
        };

        let (lobby_data, pub_tx) = {
            let lobby = lobby_arc.lock().await;
            (
                lobby.data.clone(),
                lobby.pub_tx.clone().ok_or("PubSub not initialized")?,
            )
        };

        pub_tx.send(lobby_data)?;
        Ok(())
    }

    pub async fn join_lobby(&self, lobby_id: &str, user: &Claims) -> Option<()> {
        {
            let hash_map = self.lobbies.lock().await;
            let lobby = hash_map.get(lobby_id)?;
            lobby.lock().await.join(user).await;
        }

        self.notify_lobby(lobby_id).await.ok();

        Some(())
    }

    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            lobbies: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}
