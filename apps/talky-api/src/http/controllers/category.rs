use bcrypt::verify;
use sqlx::{Pool, Postgres};
use talky_services::{
    category::service::{
        CategoryResource, CategoryService, CategoryType, CreateCategoryArgs, ListCategoryArgs,
        ListCategoryMeta,
    },
    message::service::{ListMessageArgs, ListMessageMeta, MessageResource, MessageService},
    pagination::ListResult,
    user::service::{ListUserArgs, ListUserMeta, UserResource, UserService},
};

use rspc::Router;
use serde::{Deserialize, Serialize};
use specta::Type;
use talky_auth::JwtService;
use talky_data::models::user::User;

use crate::{
    error::{AppError, AppResult},
    http::context::Ctx,
};

pub struct CategoryController {
    ctx: Ctx,
    category_service: CategoryService,
}

impl CategoryController {
    pub async fn list(
        self,
        args: ListCategoryArgs,
    ) -> AppResult<ListResult<CategoryResource, ListCategoryMeta>> {
        let response = self
            .category_service
            .list(&args)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub(crate) fn new(ctx: Ctx) -> Self {
        let category_service = CategoryService::new(ctx.pool_clone());

        Self {
            ctx,
            category_service,
        }
    }
}
