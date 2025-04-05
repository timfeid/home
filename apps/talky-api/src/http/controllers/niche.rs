use bcrypt::verify;
use sqlx::{Pool, Postgres};
use talky_services::{
    niche::service::{ListNicheArgs, ListNicheMeta, NicheResource, NicheService},
    pagination::ListResult,
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

pub struct NicheController {
    ctx: Ctx,
    niche_service: NicheService,
}

impl NicheController {
    pub async fn find_by_slug(self, slug: String) -> AppResult<NicheResource> {
        let response = self
            .niche_service
            .find_by_slug(slug)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub async fn list(
        self,
        args: ListNicheArgs,
    ) -> AppResult<ListResult<NicheResource, ListNicheMeta>> {
        let response = self
            .niche_service
            .list_for_user(args)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub(crate) fn new(ctx: Ctx) -> Self {
        let niche_service = NicheService::new(ctx.pool_clone());
        Self { ctx, niche_service }
    }
}
