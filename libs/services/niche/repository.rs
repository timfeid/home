use std::fmt::{self, Display};

use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    pagination::{Cursor, Model, Node, PaginationArgs, WithPagination},
    repository::Repository,
    DatabasePool,
};

use super::service::{ListNicheArgs, NicheResource};

pub(crate) struct NicheRepository {
    connection: DatabasePool,
}

pub(crate) struct NicheModel {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) slug: String,
}
impl NicheModel {
    pub fn new(name: String) -> Self {
        Self {
            id: name.to_lowercase(),
            slug: name.to_lowercase(),
            name,
        }
    }
}

impl Model<NicheResource> for NicheModel {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn to_node(&self) -> NicheResource {
        NicheResource {
            id: self.id.clone(),
            name: self.name.clone(),
            slug: self.slug.clone(),
        }
    }
}

impl NicheRepository {
    pub fn new(connection: DatabasePool) -> Self {
        Self { connection }
    }

    pub fn find_one(&self, slug: String) -> Result<NicheModel, sqlx::Error> {
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

        Ok(NicheModel::new(name))
    }
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct NicheCursor {
    pub id: String,
}

impl Cursor for NicheCursor {
    type CursorType = NicheCursor;

    fn encode(cursor: &NicheCursor) -> String {
        let cursor_str = cursor.to_string();
        general_purpose::STANDARD.encode(cursor_str)
    }

    fn decode(encoded: &str) -> Option<NicheCursor> {
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

impl Display for NicheCursor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Serialize the FollowCursor to JSON and write it as a string
        match serde_json::to_string(self) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, ""),
        }
    }
}

impl Repository<NicheModel, ListNicheArgs> for NicheRepository {
    async fn count(&self, args: &ListNicheArgs) -> Result<i32, sqlx::Error> {
        Ok(2)
    }

    async fn find(
        &self,
        after: Option<(
            crate::repository::CursorDirection,
            impl crate::pagination::Cursor + Send,
        )>,
        take: i32,
        args: &ListNicheArgs,
    ) -> Result<Vec<NicheModel>, sqlx::Error> {
        Ok(vec![NicheModel::new("Devils".to_string())])
    }
}
