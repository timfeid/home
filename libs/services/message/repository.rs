use std::fmt::{self, Display};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{query, query_as, types::time::PrimitiveDateTime};

use crate::{
    error::AppResult,
    pagination::{Cursor, Model, Node, PaginationArgs, WithPagination},
    repository::Repository,
    DatabasePool,
};

use super::service::{ListMessageArgs, MessageResource};

pub(crate) struct MessageRepository {
    connection: DatabasePool,
}

pub(crate) struct MessageModel {
    pub(super) id: String,
    pub(super) contents: String,
    pub(super) channel_id: String,
    pub(super) created_at: PrimitiveDateTime,
    pub(super) user_id: String,
}

impl Model<MessageResource> for MessageModel {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn to_node(&self) -> MessageResource {
        let timestamp = (self.created_at.assume_utc().unix_timestamp() * 1000);
        MessageResource {
            id: self.id.clone(),
            user_id: self.user_id.clone(),
            timestamp: timestamp as u64,
            contents: self.contents.clone(),
        }
    }
}

impl MessageRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }

    pub async fn add_chat_message(&self, channel_id: String, message: &MessageResource) {
        let id = ulid::Ulid::new().to_string();
        query!(
            "insert into messages (id, contents, channel_id, user_id) values ($1, $2, $3, $4)",
            id,
            message.contents,
            channel_id,
            message.user_id,
        )
        .execute(self.connection.as_ref())
        .await
        .ok();
    }
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct MessageCursor {
    pub id: String,
}

impl Cursor for MessageCursor {
    type CursorType = MessageCursor;

    fn encode(cursor: &MessageCursor) -> String {
        let cursor_str = cursor.to_string();
        general_purpose::STANDARD.encode(cursor_str)
    }

    fn decode(encoded: &str) -> Option<MessageCursor> {
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

impl Display for MessageCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the FollowCursor to JSON and write it as a string
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, ""),
        }
    }
}

impl Repository<MessageModel, ListMessageArgs> for MessageRepository {
    async fn count(&self, args: &ListMessageArgs) -> AppResult<i32> {
        Ok(2)
    }

    async fn find(
        &self,
        after: Option<(
            crate::repository::CursorDirection,
            impl crate::pagination::Cursor + Send,
        )>,
        take: i32,
        args: &ListMessageArgs,
    ) -> AppResult<Vec<MessageModel>> {
        let messages = query_as!(
            MessageModel,
            "select
                id,
                contents,
                channel_id,
                created_at,
                user_id
            from messages where channel_id = $1",
            args.channel_id
        )
        .fetch_all(self.connection.as_ref())
        .await?;
        Ok(messages)
    }
}
