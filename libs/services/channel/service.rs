use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    pagination::{
        connection_from_repository, Cursor, ListResult, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{ChannelCursor, ChannelRepository};

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
    pub async fn list_for_user(
        &self,
        args: ListChannelArgs,
    ) -> Result<ListResult<ChannelResource, ListChannelMeta>, sqlx::Error> {
        connection_from_repository(&args, self.repository.clone()).await
    }

    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(ChannelRepository::new(pool)),
        }
    }
}

mod tests {
    use std::sync::Arc;

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
                .list_for_user(ListChannelArgs {
                    before: None,
                    after: None,
                    first: None,
                    last: None,
                    niche_id: "".to_string()
                })
                .await
        );
    }
}
