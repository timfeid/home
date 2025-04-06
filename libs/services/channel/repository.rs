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
    pagination::{Cursor, Model},
    repository::Repository,
    DatabasePool,
};

use super::service::{ChannelResource, ChannelType, CreateChannelArgs, ListChannelArgs};

pub(crate) struct ChannelRepository {
    connection: DatabasePool,
}

pub(crate) struct ChannelModel {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) r#type: ChannelType,
    pub(super) niche_id: String,
    pub(super) is_temporary: bool,
}

impl ChannelModel {
    pub fn new(name: String) -> Self {
        Self {
            id: name.to_lowercase(),
            slug: name.to_lowercase(),
            r#type: if &name == "Gameday" {
                ChannelType::Chat
            } else if &name == "News" {
                ChannelType::Feed
            } else {
                ChannelType::MultiMedia
            },
            name,
            niche_id: "".to_string(),
            is_temporary: false,
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
            id: self.id.clone(),
            slug: self.slug.clone(),
            r#type: self.r#type.clone(),
            is_temporary: self.is_temporary.clone(),
        }
    }
}

impl ChannelRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }

    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<ChannelModel>, sqlx::Error> {
        Ok(vec![
            ChannelModel::new("News".to_string()),
            ChannelModel::new("Gameday".to_string()),
        ])
    }

    pub async fn find_by_slug(&self, slug: String) -> Result<ChannelModel, sqlx::Error> {
        let name = slug
            .chars()
            .enumerate()
            .map(|(i, c)| {
                if i == 0 {
                    c.to_uppercase().collect::<String>()
                } else {
                    c.to_string()
                }
            })
            .collect::<String>();
        Ok(ChannelModel::new(name))
    }

    pub async fn create(&self, args: &CreateChannelArgs) -> Result<ChannelModel, sqlx::Error> {
        let id = ulid::Ulid::new().to_string();
        let slug = slugify!(&args.name);

        query_as!(
                    ChannelModel,
                    "insert into channels (id, name, slug, type, niche_id, is_temporary) values ($1, $2, $3, $4, $5, true) returning id, name, is_temporary, slug, type as \"type: ChannelType\", niche_id",

                    id,
                    args.name,
                    slug,
                    args.r#type as ChannelType,
                    args.niche_id,
                )
                .fetch_one(self.connection.as_ref())
                .await
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
        let row = query!(
            r#"select
                count(*) as count
                from channels where niche_id = $1"#,
            args.niche_id
        )
        .fetch_one(self.connection.as_ref())
        .await?;

        Ok(row.count.unwrap_or_else(|| 0).try_into().unwrap())
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
        query_as!(
            ChannelModel,
            r#"select
                id,
                name,
                slug,
                type as "type: ChannelType",
                niche_id,
                is_temporary
                from channels where niche_id = $1"#,
            args.niche_id
        )
        .fetch_all(self.connection.as_ref())
        .await
    }
}
