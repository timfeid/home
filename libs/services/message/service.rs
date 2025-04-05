use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    pagination::{
        connection_from_repository, Cursor, ListResult, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{MessageCursor, MessageRepository};

#[derive(Type, Serialize, Deserialize, Default, Debug)]
pub struct ListMessageMeta {}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct ListMessageArgs {
    pub before: Option<String>,
    pub after: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
    pub channel_id: String,
}

impl WithPagination for ListMessageArgs {
    fn pagination(&self) -> PaginationArgs {
        PaginationArgs {
            before: self.before.clone(),
            after: self.after.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        }
    }

    type Meta = ListMessageMeta;
    type CursorType = MessageCursor;

    fn get_meta(&self) -> Self::Meta {
        ListMessageMeta {}
    }

    fn to_cursor(&self, id: String) -> Self::CursorType {
        let mut cursor =
            MessageCursor::decode(&self.after.as_ref().map_or("", |v| v)).unwrap_or_default();
        cursor.id = id;
        cursor
    }
}

pub struct MessageService {
    repository: Arc<MessageRepository>,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct AddChatMessageArgs {
    pub channel_id: String,
    pub user_id: String,
    pub contents: String,
}

#[derive(Type, Serialize, Debug, Clone)]
pub struct MessageResource {
    pub id: String,
    pub user_id: String,
    pub timestamp: u64,
    pub contents: String,
    // category_tree: Vec<String>,
}

impl Node for MessageResource {
    fn id(&self) -> String {
        self.id.clone()
    }
}

impl MessageResource {
    fn from_args(args: AddChatMessageArgs) -> MessageResource {
        let id = ulid::Ulid::new().to_string();
        MessageResource {
            id,
            user_id: args.user_id,
            timestamp: (Utc::now().timestamp() * 1000) as u64,
            contents: args.contents,
        }
    }
}

impl MessageService {
    pub async fn list(
        &self,
        args: ListMessageArgs,
    ) -> Result<ListResult<MessageResource, ListMessageMeta>, sqlx::Error> {
        connection_from_repository(&args, self.repository.clone()).await
    }

    pub async fn add_chat_message(&self, args: AddChatMessageArgs) -> MessageResource {
        let channel_id = args.channel_id.clone();
        let resource = MessageResource::from_args(args);
        self.repository
            .add_chat_message(channel_id, &resource)
            .await;

        resource
    }

    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(MessageRepository::new(pool)),
        }
    }
}

mod tests {
    use std::sync::Arc;

    use talky_data::database::create_connection;

    use crate::{
        channel::service::{ChannelService, ListChannelArgs},
        message::service::{ListMessageArgs, MessageService},
        DatabasePool,
    };

    #[tokio::test]
    async fn test() {
        let url = "postgresql://postgres:wat@0.0.0.0/gangsta";
        let pool: DatabasePool = create_connection(url).await;
        let channel_service = MessageService::new(pool);
        println!(
            "{:?}",
            channel_service
                .list(ListMessageArgs {
                    before: None,
                    after: None,
                    first: None,
                    last: None,
                    channel_id: "".to_string()
                })
                .await
        );
    }
}
