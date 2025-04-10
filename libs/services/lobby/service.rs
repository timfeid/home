use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    error::AppResult,
    pagination::{
        connection_from_repository, Cursor, ListResult, Model, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{LobbyCursor, LobbyModel, LobbyRepository};

#[derive(Type, Serialize, Deserialize, Default, Debug)]
pub struct ListLobbyMeta {}

impl WithPagination for ListLobbyArgs {
    fn pagination(&self) -> PaginationArgs {
        PaginationArgs {
            before: self.before.clone(),
            after: self.after.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        }
    }

    type Meta = ListLobbyMeta;
    type CursorType = LobbyCursor;

    fn get_meta(&self) -> Self::Meta {
        ListLobbyMeta {}
    }

    fn to_cursor(&self, id: String) -> Self::CursorType {
        let mut cursor =
            LobbyCursor::decode(&self.after.as_ref().map_or("", |v| v)).unwrap_or_default();
        cursor.id = id;
        cursor
    }
}

pub struct LobbyService {
    repository: Arc<LobbyRepository>,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct ListLobbyArgs {
    pub before: Option<String>,
    pub after: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub niche_id: String,
}

pub enum TemporaryContract {
    ExpireWhenEmpty,
    Expires { expires: i32 },
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct CreateLobbyArgs {
    pub name: String,
    pub channel_id: String,
}

#[derive(Type, Serialize, Debug, Clone)]
pub struct LobbyResource {
    pub id: String,
    pub name: String,
    pub channel_id: String,
    pub niche_id: String,
    pub owner_user_id: String,
}

#[derive(PartialEq, sqlx::Type, Type, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "lobby_type", rename_all = "snake_case")]
pub enum LobbyType {
    Chat,
    Feed,
    MultiMedia,
}

impl Node for LobbyResource {
    fn id(&self) -> String {
        self.name.clone()
    }
}

impl LobbyService {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(LobbyRepository::new(pool)),
        }
    }

    pub async fn create(&self, args: &CreateLobbyArgs, user_id: &str) -> AppResult<LobbyResource> {
        Ok(self.repository.create(args, user_id).await?.to_node())
    }

    pub async fn find_by_id(&self, id: String) -> AppResult<LobbyResource> {
        Ok(self.repository.find_by_id(id).await?.to_node())
    }
}
