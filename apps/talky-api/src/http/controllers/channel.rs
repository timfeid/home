use bcrypt::verify;
use sqlx::{Pool, Postgres};
use talky_services::{
    channel::service::{ChannelResource, ChannelService, ListChannelArgs, ListChannelMeta},
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

pub struct ChannelController {
    ctx: Ctx,
    channel_service: ChannelService,
}

impl ChannelController {
    pub async fn list_in(self) -> AppResult<ListResult<ChannelResource, ListChannelMeta>> {
        let user = self.ctx.required_user()?;
        let response = self
            .channel_service
            .list_for_user(ListChannelArgs {
                before: None,
                after: None,
                first: None,
                last: None,
                niche_id: "temp_niche_id".to_string(),
            })
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub(crate) fn new(ctx: Ctx) -> Self {
        let channel_service = ChannelService::new(ctx.pool_clone());
        Self {
            ctx,
            channel_service,
        }
    }
}
