use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    pagination::{
        connection_from_repository, Cursor, ListResult, Model, Node, PaginationArgs, WithPagination,
    },
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

#[derive(Type, Serialize, Debug)]
pub struct ChannelResource {
    pub id: String,
    pub name: String,
    pub slug: String,
    // category_tree: Vec<String>,
}

impl Node for ChannelResource {
    fn id(&self) -> String {
        self.name.clone()
    }
}

impl ChannelService {
    pub async fn list_for_user(&self, user_id: &str) -> Result<Vec<ChannelResource>, sqlx::Error> {
        Ok(self
            .repository
            .list_for_user(user_id)
            .await?
            .iter()
            .map(|channel| channel.to_node())
            .collect())
    }

    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(ChannelRepository::new(pool)),
        }
    }

    pub async fn find_by_slug(&self, slug: String) -> Result<ChannelResource, sqlx::Error> {
        Ok(self.repository.find_by_slug(slug).await?.to_node())
    }
}

mod tests {
    use talky_data::database::create_connection;

    use crate::{
        channel::service::{ChannelService, ListChannelArgs},
        DatabasePool,
    };

    #[tokio::test]
    async fn test() {
        let url = "postgresql://postgres:wat@0.0.0.0/gangsta";
        let pool: DatabasePool = create_connection(url).await;
        let channel_service = ChannelService::new(pool);
        println!("{:?}", channel_service.list_for_user(&"tim").await);
    }
}
