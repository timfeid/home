use bcrypt::verify;
use sqlx::{Pool, Postgres};
use talky_services::{
    lobby::service::{
        CreateLobbyArgs, ListLobbyArgs, ListLobbyMeta, LobbyResource, LobbyService, LobbyType,
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

pub struct LobbyController {
    ctx: Ctx,
    lobby_service: LobbyService,
}

impl LobbyController {
    pub async fn create(self, args: CreateLobbyArgs) -> AppResult<LobbyResource> {
        let user = self.ctx.required_user()?;
        let response = self
            .lobby_service
            .create(&args, &user.sub)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(response)
    }

    pub(crate) fn new(ctx: Ctx) -> Self {
        let lobby_service = LobbyService::new(ctx.pool_clone());

        Self { ctx, lobby_service }
    }
}
