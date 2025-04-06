use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
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

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct CreateChannelArgs {
    pub name: String,
    pub niche_id: String,
    pub r#type: ChannelType,
    // pub expire_contract: Option<String>,
}

#[derive(Type, Serialize, Debug)]
pub struct ChannelResource {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub r#type: ChannelType,
    pub is_temporary: bool,
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

impl Node for ChannelResource {
    fn id(&self) -> String {
        self.name.clone()
    }
}

impl ChannelService {
    pub async fn list(
        &self,
        args: &ListChannelArgs,
    ) -> Result<ListResult<ChannelResource, ListChannelMeta>, sqlx::Error> {
        connection_from_repository(args, self.repository.clone()).await
    }

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

    pub async fn create(&self, args: &CreateChannelArgs) -> Result<ChannelResource, sqlx::Error> {
        Ok(self.repository.create(args).await?.to_node())
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
        println!(
            "{:?}",
            channel_service
                .list(&ListChannelArgs {
                    before: None,
                    after: None,
                    first: None,
                    last: None,
                    niche_id: "devils".to_owned()
                })
                .await
        );
    }
}
