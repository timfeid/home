use bcrypt::verify;
use sqlx::{Pool, Postgres};
use talky_services::{
    channel::service::{ChannelResource, ChannelService, ListChannelArgs, ListChannelMeta},
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

pub struct ChannelController {
    ctx: Ctx,
    channel_service: ChannelService,
    user_service: UserService,
    message_service: MessageService,
}

impl ChannelController {
    pub async fn list(
        self,
        args: ListChannelArgs,
    ) -> AppResult<ListResult<ChannelResource, ListChannelMeta>> {
        let response = self
            .channel_service
            .list(&args)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub async fn find_by_slug(self, slug: String) -> AppResult<ChannelResource> {
        let response = self
            .channel_service
            .find_by_slug(slug)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub async fn list_messages(
        self,
        args: ListMessageArgs,
    ) -> AppResult<ListResult<MessageResource, ListMessageMeta>> {
        let response = self
            .message_service
            .list(args)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub async fn list_users(
        self,
        args: ListUserArgs,
    ) -> AppResult<ListResult<UserResource, ListUserMeta>> {
        let response = self
            .user_service
            .list(&args)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub async fn list_in(self) -> AppResult<Vec<ChannelResource>> {
        let user = self.ctx.required_user()?;
        let response = self
            .channel_service
            .list_for_user(&user.sub)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub(crate) fn new(ctx: Ctx) -> Self {
        let channel_service = ChannelService::new(ctx.pool_clone());
        let user_service = UserService::new(ctx.pool_clone());
        let message_service = MessageService::new(ctx.pool_clone());

        Self {
            ctx,
            channel_service,
            user_service,
            message_service,
        }
    }
}
