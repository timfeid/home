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
    lobby::repository::LobbyModel,
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
    pub(super) lobbies: Vec<LobbyModel>,
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
            lobbies: self.lobbies.iter().map(|lobby| lobby.to_node()).collect(),
        }
    }
}

impl ChannelRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }

    pub async fn find_by_id(&self, id: String) -> AppResult<ChannelModel> {
        self.find_one_by("id", id).await
    }

    pub async fn find_by_slug(&self, slug: String) -> AppResult<ChannelModel> {
        self.find_one_by("slug", slug).await
    }

    async fn find_one_by(&self, column: &str, value: String) -> AppResult<ChannelModel> {
        let query_string = format!(
            r#"select
                    id,
                    name,
                    slug,
                    type as "type: ChannelType",
                    category_id,
                    coalesce((select niche_id from categories where id = category_id), '') as "niche_id!",

                    coalesce((select json_agg(json_build_object('id', id, 'name', name, 'channel_id', channel_id, 'owner_user_id', owner_user_id)) from lobbies where lobbies.channel_id = channels.id), '[]'::json)
 as lobbies
                    from channels where {} = $1"#,
            column
        );

        let row: (
            String,
            String,
            String,
            ChannelType,
            String,
            String,
            Option<serde_json::Value>,
        ) = query_as(&query_string)
            .bind(value)
            .fetch_one(self.connection.as_ref())
            .await
            .map_err(ServicesError::from)?;

        let (id, name, slug, r#type, category_id, niche_id, json_lobbies) = row;

        // Manually convert JsonValue into Vec<LobbyModel>
        let lobbies: Vec<LobbyModel> = match json_lobbies {
            Some(json_value) => serde_json::from_value(json_value).unwrap_or_default(),
            None => vec![],
        };

        Ok(ChannelModel {
            id,
            name,
            slug,
            r#type,
            category_id,
            niche_id,
            lobbies,
        })
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
