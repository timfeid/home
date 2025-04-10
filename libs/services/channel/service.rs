use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    error::AppResult,
    lobby::service::LobbyResource,
    niche::service::NicheResource,
    pagination::{
        connection_from_repository, Cursor, ListResult, Model, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{ChannelCursor, ChannelModel, ChannelRepository};

#[derive(Type, Serialize, Deserialize, Default, Debug)]
pub struct ListChannelMeta {}

impl WithPagination for ListChannelArgs {
    fn pagination(&self) -> PaginationArgs {
        PaginationArgs {
            before: self.before.clone(),
            after: self.after.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        }
    }

    type Meta = ListChannelMeta;
    type CursorType = ChannelCursor;

    fn get_meta(&self) -> Self::Meta {
        ListChannelMeta {}
    }

    fn to_cursor(&self, id: String) -> Self::CursorType {
        let mut cursor =
            ChannelCursor::decode(&self.after.as_ref().map_or("", |v| v)).unwrap_or_default();
        cursor.id = id;
        cursor
    }
}

pub struct ChannelService {
    repository: Arc<ChannelRepository>,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct ListChannelArgs {
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

#[derive(Type, Serialize, Debug, Clone)]
pub struct ChannelResource {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub r#type: ChannelType,
    pub niche_id: String,
    pub lobbies: Vec<LobbyResource>,
    // category_tree: Vec<String>,
}

#[derive(Type, Serialize, Debug, Clone)]
pub struct ChannelCategoryResource {
    pub id: String,
    pub name: String,
    pub niche_id: String,
    pub channels: Vec<ChannelResource>,
    // category_tree: Vec<String>,
}

#[derive(PartialEq, sqlx::Type, Type, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "channel_type", rename_all = "snake_case")]
pub enum ChannelType {
    Chat,
    Feed,
    MultiMedia,
}

impl Node for ChannelCategoryResource {
    fn id(&self) -> String {
        self.name.clone()
    }
}

impl ChannelService {
    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(ChannelRepository::new(pool)),
        }
    }

    pub async fn find_by_slug(&self, slug: String) -> AppResult<ChannelResource> {
        Ok(self.repository.find_by_slug(slug).await?.to_node())
    }

    pub async fn find_by_id(&self, id: String) -> AppResult<ChannelResource> {
        Ok(self.repository.find_by_id(id).await?.to_node())
    }
}
