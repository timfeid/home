use std::{
    collections::HashMap,
    fmt::{self, Display},
    sync::RwLock,
};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use slugify::slugify;
use specta::Type;
use sqlx::{query, query_as};

use crate::{
    error::{AppResult, ServicesError},
    pagination::{Cursor, Model},
    repository::Repository,
    DatabasePool,
};

use super::service::{CreateLobbyArgs, ListLobbyArgs, LobbyResource, LobbyType};

pub(crate) struct LobbyRepository {
    connection: DatabasePool,
}

pub(crate) struct LobbyModel {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) channel_id: String,
    pub(super) owner_user_id: String,
}

impl Model<LobbyResource> for LobbyModel {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn to_node(&self) -> LobbyResource {
        LobbyResource {
            id: self.id.clone(),
            name: self.name.clone(),
            channel_id: self.channel_id.clone(),
            owner_user_id: self.owner_user_id.clone(),
        }
    }
}

impl LobbyRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }

    pub async fn find_by_id(&self, id: String) -> AppResult<LobbyModel> {
        query_as!(
            LobbyModel,
            r#"select
                id,
                name,
                channel_id,
                owner_user_id
                from lobbies where id = $1"#,
            id
        )
        .fetch_one(self.connection.as_ref())
        .await
        .map_err(ServicesError::from)
    }

    pub async fn create(
        &self,
        args: &CreateLobbyArgs,
        owner_user_id: &str,
    ) -> AppResult<LobbyModel> {
        let id = ulid::Ulid::new().to_string();

        query_as!(
                    LobbyModel,
                    "insert into lobbies (id, name, channel_id, owner_user_id) values ($1, $2, $3, $4) returning id, name, channel_id, owner_user_id",
                    id,
                    args.name,
                    args.channel_id,
                    owner_user_id,
                )
                .fetch_one(self.connection.as_ref())
                .await.map_err(ServicesError::from)
    }
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LobbyCursor {
    pub id: String,
}

impl Cursor for LobbyCursor {
    type CursorType = LobbyCursor;

    fn encode(cursor: &LobbyCursor) -> String {
        let cursor_str = cursor.to_string();
        general_purpose::STANDARD.encode(cursor_str)
    }

    fn decode(encoded: &str) -> Option<LobbyCursor> {
        let decoded_bytes = general_purpose::STANDARD.decode(encoded).ok()?;
        let decoded_str = String::from_utf8(decoded_bytes).ok()?;
        serde_json::from_str(&decoded_str).ok()
    }

    fn sort_key(&self) -> String {
        String::default()
    }

    fn id(&self) -> String {
        self.id.clone()
    }
}

impl Display for LobbyCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the FollowCursor to JSON and write it as a string
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, ""),
        }
    }
}
