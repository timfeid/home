use std::sync::Arc;

use rspc::Router;
use talky_services::{
    lobby::service::{CreateLobbyArgs, ListLobbyArgs},
    message::service::ListMessageArgs,
    user::service::ListUserArgs,
};

use crate::http::{
    context::Ctx,
    controllers::{
        authentication::{AuthenticationController, LoginArgs},
        lobby::LobbyController,
    },
};

use super::BaseProcedure;

pub fn create_lobby_router() -> Router<Ctx> {
    Router::<Ctx>::new().procedure("lobby_create_temporary", {
        <BaseProcedure>::builder()
            .query(|ctx, args: CreateLobbyArgs| LobbyController::new(ctx).create(args))
    })
}
