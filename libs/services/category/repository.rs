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
    channel::repository::ChannelModel,
    error::{AppResult, ServicesError},
    pagination::{Cursor, Model},
    repository::Repository,
    DatabasePool,
};

use super::service::{CategoryResource, CategoryType, CreateCategoryArgs, ListCategoryArgs};

pub(crate) struct CategoryRepository {
    connection: DatabasePool,
}

pub(crate) struct CategoryModel {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) niche_id: String,
    pub(super) channels: Vec<ChannelModel>,
}

impl Model<CategoryResource> for CategoryModel {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn to_node(&self) -> CategoryResource {
        CategoryResource {
            id: self.id.clone(),
            name: self.name.clone(),
            niche_id: self.niche_id.clone(),
            channels: self
                .channels
                .iter()
                .map(|channel| channel.to_node())
                .collect(),
        }
    }
}

impl CategoryRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct CategoryCursor {
    pub id: String,
}

impl Cursor for CategoryCursor {
    type CursorType = CategoryCursor;

    fn encode(cursor: &CategoryCursor) -> String {
        let cursor_str = cursor.to_string();
        general_purpose::STANDARD.encode(cursor_str)
    }

    fn decode(encoded: &str) -> Option<CategoryCursor> {
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

impl Display for CategoryCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the FollowCursor to JSON and write it as a string
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, ""),
        }
    }
}

impl CategoryRepository {
    pub async fn find_by_id(&self, id: &str) -> AppResult<CategoryModel> {
        let row = query!(
                r#"select
                        id,
                        name,
                        niche_id,
                        (select json_agg(json_build_object(
                                        'id', id,
                                        'name', name,
                                        'slug', slug,
                                        'type', type,
                                        'category_id', category_id,
                                        'is_temporary', is_temporary
                                        )) from channels where category_id = categories.id) as channels
                        from categories where id = $1"#,
                        id
            )
            .fetch_one(self.connection.as_ref())
            .await
            .map_err(ServicesError::from)?;

        let category = CategoryModel {
            id: row.id,
            name: row.name,
            niche_id: row.niche_id,
            channels: serde_json::from_value(row.channels.unwrap_or_default()).unwrap_or_default(),
        };

        Ok(category)
    }
}

impl Repository<CategoryModel, ListCategoryArgs> for CategoryRepository {
    async fn count(&self, args: &ListCategoryArgs) -> AppResult<i32> {
        let row = query!(
            r#"select
                count(*) as count
                from categories where niche_id = $1"#,
            args.niche_id
        )
        .fetch_one(self.connection.as_ref())
        .await?;

        Ok(row.count.unwrap_or_else(|| 0).try_into().unwrap())
    }

    async fn find(
        &self,
        _after: Option<(
            crate::repository::CursorDirection,
            impl crate::pagination::Cursor + Send,
        )>,
        _take: i32,
        args: &ListCategoryArgs,
    ) -> AppResult<Vec<CategoryModel>> {
        let rows = query!(
            r#"
            SELECT
                id,
                name,
                niche_id,
                (
                    SELECT json_agg(json_build_object(
                        'id', id,
                        'name', name,
                        'slug', slug,
                        'type', type,
                        'category_id', category_id,
                        'niche_id', niche_id,
                        'lobbies',
                        COALESCE(
                            (
                                SELECT json_agg(json_build_object(
                                    'id', id,
                                    'name', name,
                                    'channel_id', channel_id,
                                    'owner_user_id', owner_user_id,
                                    'niche_id', niche_id
                                ))
                                FROM lobbies
                                WHERE lobbies.channel_id = channels.id
                            ),
                            '[]'::json
                        )
                    ))
                    FROM channels
                    WHERE category_id = categories.id
                ) AS channels
            FROM categories
            WHERE niche_id = $1
            "#,
            args.niche_id
        )
        .fetch_all(self.connection.as_ref())
        .await
        .map_err(ServicesError::from)?;

        let categories = rows
            .into_iter()
            .map(|row| {
                let channels_value = row.channels.as_ref().unwrap_or(&serde_json::Value::Null);
                let channels = serde_json::from_value(channels_value.clone()).map_err(|e| {
                    ServicesError::SQLError(format!(
                        "Failed to unwrap channel details: {:?}\nrow:{:?}",
                        e, row
                    ))
                })?;
                Ok(CategoryModel {
                    id: row.id,
                    name: row.name,
                    niche_id: row.niche_id,
                    channels,
                })
            })
            .collect::<Result<Vec<_>, ServicesError>>()?;

        Ok(categories)
    }
}
