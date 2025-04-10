use std::sync::Arc;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    channel::service::ChannelResource,
    error::AppResult,
    pagination::{
        connection_from_repository, Cursor, ListResult, Model, Node, PaginationArgs, WithPagination,
    },
    repository::Repository,
    DatabasePool,
};

use super::repository::{CategoryCursor, CategoryModel, CategoryRepository};

#[derive(Type, Serialize, Deserialize, Default, Debug)]
pub struct ListCategoryMeta {}

impl WithPagination for ListCategoryArgs {
    fn pagination(&self) -> PaginationArgs {
        PaginationArgs {
            before: self.before.clone(),
            after: self.after.clone(),
            first: self.first.clone(),
            last: self.last.clone(),
        }
    }

    type Meta = ListCategoryMeta;
    type CursorType = CategoryCursor;

    fn get_meta(&self) -> Self::Meta {
        ListCategoryMeta {}
    }

    fn to_cursor(&self, id: String) -> Self::CursorType {
        let mut cursor =
            CategoryCursor::decode(&self.after.as_ref().map_or("", |v| v)).unwrap_or_default();
        cursor.id = id;
        cursor
    }
}

pub struct CategoryService {
    repository: Arc<CategoryRepository>,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct ListCategoryArgs {
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
pub struct CreateCategoryArgs {
    pub name: String,
    pub niche_id: String,
    pub r#type: CategoryType,
    // pub expire_contract: Option<String>,
}

#[derive(Type, Serialize, Debug, Clone)]
pub struct CategoryResource {
    pub id: String,
    pub name: String,
    pub niche_id: String,
    pub channels: Vec<ChannelResource>,
    // category_tree: Vec<String>,
}

#[derive(PartialEq, sqlx::Type, Type, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "channel_type", rename_all = "snake_case")]
pub enum CategoryType {
    Chat,
    Feed,
    MultiMedia,
}

impl Node for CategoryResource {
    fn id(&self) -> String {
        self.name.clone()
    }
}

impl CategoryService {
    pub async fn list(
        &self,
        args: &ListCategoryArgs,
    ) -> AppResult<ListResult<CategoryResource, ListCategoryMeta>> {
        connection_from_repository(args, self.repository.clone()).await
    }

    // pub async fn list_for_user(&self, user_id: &str) -> AppResult<Vec<CategoryResource>> {
    //     Ok(self
    //         .repository
    //         .list_for_user(user_id)
    //         .await?
    //         .iter()
    //         .map(|channel| channel.to_node())
    //         .collect())
    // }

    pub fn new(pool: DatabasePool) -> Self {
        Self {
            repository: Arc::new(CategoryRepository::new(pool)),
        }
    }

    pub async fn find_by_id(&self, id: String) -> AppResult<CategoryResource> {
        Ok(self.repository.find_by_id(&id).await?.to_node())
    }
}

mod tests {
    use talky_data::database::create_connection;

    use crate::{
        category::service::{CategoryService, ListCategoryArgs},
        DatabasePool,
    };

    #[tokio::test]
    async fn test() {
        let url = "postgresql://postgres:wat@0.0.0.0/gangsta";
        let pool: DatabasePool = create_connection(url).await;
        let channel_service = CategoryService::new(pool);
        println!(
            "{:?}",
            channel_service
                .list(&ListCategoryArgs {
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
