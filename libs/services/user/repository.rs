use std::fmt::{self, Display};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    error::AppResult,
    pagination::{Cursor, Model, Node, PaginationArgs, WithPagination},
    repository::Repository,
    DatabasePool,
};

use super::service::{ListUserArgs, UserResource};

pub(crate) struct UserRepository {
    connection: DatabasePool,
}

pub(crate) struct UserModel {
    pub(super) id: String,
    pub(super) username: String,
    pub(super) avatar_url: Option<String>,
    pub(super) email: Option<String>,
    pub(super) password: Option<String>,
}

impl UserModel {
    pub fn new(id: String, username: String) -> Self {
        Self {
            id,
            username,
            avatar_url: None,
            email: Some("tim@timfeid.com".to_owned()),
            password: Some(
                "$2b$04$cd5jDDLNGsZ09QzNJ1vuKeol/rhqy0oGfV.aJvo/eOQfVzapKJyN6".to_owned(),
            ),
        }
    }
}

impl Model<UserResource> for UserModel {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn to_node(&self) -> UserResource {
        UserResource {
            id: self.id.clone(),
            username: self.username.clone(),
            avatar_url: self.avatar_url.clone(),
        }
    }
}

impl UserRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct UserCursor {
    pub id: String,
}

impl Cursor for UserCursor {
    type CursorType = UserCursor;

    fn encode(cursor: &UserCursor) -> String {
        let cursor_str = cursor.to_string();
        general_purpose::STANDARD.encode(cursor_str)
    }

    fn decode(encoded: &str) -> Option<UserCursor> {
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

impl Display for UserCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the FollowCursor to JSON and write it as a string
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, ""),
        }
    }
}

impl Repository<UserModel, ListUserArgs> for UserRepository {
    async fn count(&self, args: &ListUserArgs) -> AppResult<i32> {
        Ok(2)
    }

    async fn find(
        &self,
        after: Option<(
            crate::repository::CursorDirection,
            impl crate::pagination::Cursor + Send,
        )>,
        take: i32,
        args: &ListUserArgs,
    ) -> AppResult<Vec<UserModel>> {
        Ok(vec![
            UserModel::new("dazed".to_string(), "dazed".to_string()),
            UserModel::new("jimbo".to_string(), "jimbo".to_string()),
            UserModel::new("africkuh".to_string(), "africkUh".to_string()),
        ])
    }
}
