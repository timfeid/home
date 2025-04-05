use std::fmt::{self, Display};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    pagination::{Cursor, Model, Node, PaginationArgs, WithPagination},
    repository::Repository,
    DatabasePool,
};

use super::service::{ChannelResource, ListChannelArgs};

pub(crate) struct ChannelRepository {
    connection: DatabasePool,
}

pub(crate) struct ChannelModel {
    pub(super) name: String,
    pub(super) slug: String,
}
impl ChannelModel {
    pub fn new(name: String) -> Self {
        Self {
            slug: name.to_lowercase(),
            name,
        }
    }
}

impl Model<ChannelResource> for ChannelModel {
    fn id(&self) -> String {
        self.slug.clone()
    }

    fn to_node(&self) -> ChannelResource {
        ChannelResource {
            name: self.name.clone(),
            slug: self.slug.clone(),
        }
    }
}

impl ChannelRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct ChannelCursor {
    pub id: String,
}

impl Cursor for ChannelCursor {
    type CursorType = ChannelCursor;

    fn encode(cursor: &ChannelCursor) -> String {
        let cursor_str = cursor.to_string();
        general_purpose::STANDARD.encode(cursor_str)
    }

    fn decode(encoded: &str) -> Option<ChannelCursor> {
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

impl Display for ChannelCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the FollowCursor to JSON and write it as a string
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, ""),
        }
    }
}

impl Repository<ChannelModel, ListChannelArgs> for ChannelRepository {
    async fn count(&self, args: &ListChannelArgs) -> Result<i32, sqlx::Error> {
        Ok(2)
    }

    async fn find(
        &self,
        after: Option<(
            crate::repository::CursorDirection,
            impl crate::pagination::Cursor + Send,
        )>,
        take: i32,
        args: &ListChannelArgs,
    ) -> Result<Vec<ChannelModel>, sqlx::Error> {
        Ok(vec![
            ChannelModel::new("News".to_string()),
            ChannelModel::new("Gameday".to_string()),
        ])
    }
}
