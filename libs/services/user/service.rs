use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    error::AppResult,
    pagination::{
        connection_from_repository, Cursor, ListResult, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{UserCursor, UserRepository};

#[derive(Type, Serialize, Deserialize, Default, Debug)]
pub struct ListUserMeta {}

impl WithPagination for ListUserArgs {
    fn pagination(&self) -> PaginationArgs {
        PaginationArgs {
            before: self.before.clone(),
            after: self.after.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        }
    }

    type Meta = ListUserMeta;
    type CursorType = UserCursor;

    fn get_meta(&self) -> Self::Meta {
        ListUserMeta {}
    }

    fn to_cursor(&self, id: String) -> Self::CursorType {
        let mut cursor =
            UserCursor::decode(&self.after.as_ref().map_or("", |v| v)).unwrap_or_default();
        cursor.id = id;
        cursor
    }
}

pub struct UserService {
    repository: Arc<UserRepository>,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct ListUserArgs {
    pub before: Option<String>,
    pub after: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub niche_id: String,
}

#[derive(Type, Serialize, Debug)]
pub struct UserResource {
    pub id: String,
    pub username: String,
    pub avatar_url: Option<String>,
}

impl Node for UserResource {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl UserService {
    pub async fn list(
        &self,
        args: &ListUserArgs,
    ) -> AppResult<ListResult<UserResource, ListUserMeta>> {
        connection_from_repository(args, self.repository.clone()).await
    }

    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(UserRepository::new(pool)),
        }
    }
}

mod tests {
    use std::sync::Arc;

    use talky_data::database::create_connection;

    use crate::{
        channel::service::{ChannelService, ListChannelArgs},
        user::service::{ListUserArgs, UserService},
        DatabasePool,
    };

    #[tokio::test]
    async fn test() {
        let url = "postgresql://postgres:wat@0.0.0.0/gangsta";
        let pool: DatabasePool = create_connection(url).await;
        let args = ListUserArgs {
            before: None,
            after: None,
            first: None,
            last: None,
            niche_id: "".to_string(),
        };
        let channel_service = UserService::new(pool);
        println!("{:?}", channel_service.list(&args).await);
    }
}
