use std::{
    borrow::BorrowMut, collections::HashMap, future::Future, pin::Pin, sync::Arc, thread::Thread,
    vec,
};

use futures::StreamExt;

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct LobbyChat {
    user_id: String,
    message: String,
}
impl LobbyChat {
    pub fn new(user_id: String, message: String) -> Self {
        Self { user_id, message }
    }
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct PresenceMember {
    pub user_id: String,
    pub last_pong: u32,
    pub last_ping: u32,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct LobbyData {
    pub presence: HashMap<String, PresenceMember>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum LobbyCommandAudience {
    All,
    Socket(String),
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct LobbyCommandWrapper {
    pub command: LobbyCommand,
    pub audience: LobbyCommandAudience,
}
impl LobbyCommandWrapper {
    pub(crate) fn new_individual(command: LobbyCommand, socket_id: String) -> LobbyCommandWrapper {
        LobbyCommandWrapper {
            command,
            audience: LobbyCommandAudience::Socket(socket_id),
        }
    }

    pub(crate) fn new_all(command: LobbyCommand) -> LobbyCommandWrapper {
        LobbyCommandWrapper {
            command,
            audience: LobbyCommandAudience::All,
        }
    }
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub enum LobbyCommand {
    Ping(String),
    Data(LobbyData),
}

impl Default for LobbyData {
    fn default() -> LobbyData {
        LobbyData {
            presence: HashMap::new(),
        }
    }
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct Lobby {
    #[serde(skip_serializing, skip_deserializing)]
    pub pub_tx: Option<broadcast::Sender<LobbyCommandWrapper>>,

    pub data: LobbyData,
}

impl Lobby {}

use serde::{Deserialize, Serialize};
use specta::Type;
use talky_auth::Claims;
use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use tokio_stream::wrappers::ReceiverStream;
use ulid::Ulid;

use crate::error::{AppError, AppResult};

use super::manager::LobbyManager;

impl Lobby {
    pub async fn new(user: &Claims) -> Self {
        let (pub_tx, _) = broadcast::channel(2048);

        let mut lobby = Lobby {
            pub_tx: Some(pub_tx),
            data: LobbyData::default(),
        };

        lobby.join(user).await;

        lobby
    }

    pub async fn join(&mut self, user: &Claims) -> &mut Self {
        println!("JOIN {:?}", self);

        self
    }

    pub async fn ready(&mut self, user: &Claims) -> &mut Self {
        self
    }

    pub fn typing(&mut self, user: &Claims, message: String) -> &mut Self {
        // self.data
        //     .chat
        //     .push(LobbyChat::new(user.sub.clone(), message));

        self
    }
}

mod test {
    use std::{cell::RefCell, rc::Rc};

    use talky_auth::Claims;
    use tokio_stream::StreamExt;

    use crate::lobby::lobby::Lobby;

    #[tokio::test]
    async fn test() {
        let user_id = Claims {
            sub: "boob".to_string(),
            jti: Some("boob".to_string()),
            exp: 0,
        };
        let user_id2 = Claims {
            sub: "sakdfakjs".to_string(),
            jti: Some("asdkjfjskd".to_string()),
            exp: 0,
        };
        let lobby = &Rc::new(RefCell::new(Lobby::new(&user_id).await));

        // lobby
        //     .clone()
        //     .borrow_mut()
        //     .join(&user_id2)
        //     .await
        //     .message(&user_id2, "test".to_string());
    }
}
