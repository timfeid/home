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

use super::service::{ChannelResource, ChannelType, ListChannelArgs};

pub(crate) struct ChannelRepository {
    connection: DatabasePool,
}

#[derive(Deserialize)]
pub(crate) struct ChannelModel {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) r#type: ChannelType,
    pub(super) category_id: String,
    pub(super) niche_id: String,
}

impl Model<ChannelResource> for ChannelModel {
    fn id(&self) -> String {
        self.slug.clone()
    }

    fn to_node(&self) -> ChannelResource {
        ChannelResource {
            name: self.name.clone(),
            id: self.id.clone(),
            slug: self.slug.clone(),
            r#type: self.r#type.clone(),
            niche_id: self.niche_id.clone(),
        }
    }
}

impl ChannelRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }

    pub async fn find_by_id(&self, id: String) -> AppResult<ChannelModel> {
        query_as!(
                    ChannelModel,
                    r#"select
                        id,
                        name,
                        slug,
                        type as "type: ChannelType",
                        category_id,
                        coalesce((select niche_id from categories where id = category_id), '') as "niche_id!"
                        from channels where id = $1"#,
                    id
                )
                .fetch_one(self.connection.as_ref())
                .await
                .map_err(ServicesError::from)
    }

    pub async fn find_by_slug(&self, slug: String) -> AppResult<ChannelModel> {
        query_as!(
            ChannelModel,
            r#"select
                id,
                name,
                slug,
                type as "type: ChannelType",
                category_id,
                coalesce((select niche_id from categories where id = category_id), '') as "niche_id!"
                from channels where slug = $1"#,
            slug
        )
        .fetch_one(self.connection.as_ref())
        .await
        .map_err(ServicesError::from)
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
