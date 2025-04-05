use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    pagination::{
        connection_from_repository, Cursor, ListResult, Model, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{NicheCursor, NicheRepository};

#[derive(Type, Serialize, Deserialize, Default, Debug)]
pub struct ListNicheMeta {}

impl WithPagination for ListNicheArgs {
    fn pagination(&self) -> PaginationArgs {
        PaginationArgs {
            before: self.before.clone(),
            after: self.after.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        }
    }

    type Meta = ListNicheMeta;
    type CursorType = NicheCursor;

    fn get_meta(&self) -> Self::Meta {
        ListNicheMeta {}
    }

    fn to_cursor(&self, id: String) -> Self::CursorType {
        let mut cursor =
            NicheCursor::decode(&self.after.as_ref().map_or("", |v| v)).unwrap_or_default();
        cursor.id = id;
        cursor
    }
}

pub struct NicheService {
    repository: Arc<NicheRepository>,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct ListNicheArgs {
    pub before: Option<String>,
    pub after: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub niche_id: String,
}

#[derive(Type, Serialize, Debug)]
pub struct NicheResource {
    pub name: String,
    pub slug: String,
    pub id: String,
    // category_tree: Vec<String>,
}

impl Node for NicheResource {
    fn id(&self) -> String {
        self.name.clone()
    }
}

impl NicheService {
    pub async fn list_for_user(
        &self,
        args: ListNicheArgs,
    ) -> Result<ListResult<NicheResource, ListNicheMeta>, sqlx::Error> {
        connection_from_repository(&args, self.repository.clone()).await
    }

    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(NicheRepository::new(pool)),
        }
    }

    pub async fn find_by_slug(&self, slug: String) -> Result<NicheResource, sqlx::Error> {
        Ok(self.repository.find_one(slug)?.to_node())
    }
}

mod tests {
    use std::sync::Arc;

    use talky_data::database::create_connection;

    use crate::{
        channel::service::{ChannelService, ListChannelArgs},
        niche::service::{ListNicheArgs, NicheService},
        DatabasePool,
    };

    #[tokio::test]
    async fn test() {
        let url = "postgresql://postgres:wat@0.0.0.0/gangsta";
        let pool: DatabasePool = create_connection(url).await;
        let channel_service = NicheService::new(pool);
        println!(
            "{:?}",
            channel_service
                .list_for_user(ListNicheArgs {
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
