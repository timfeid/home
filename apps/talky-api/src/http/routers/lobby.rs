use rspc::Router;

use crate::http::context::Ctx;
use crate::http::controllers::lobby::LobbyController;

use super::BaseProcedure;

pub fn create_lobby_router() -> Router<Ctx> {
    Router::<Ctx>::new()
        .procedure("lobby_join", {
            <BaseProcedure>::builder().mutation(LobbyController::join)
        })
        .procedure("lobby_pong", {
            <BaseProcedure>::builder().mutation(LobbyController::pong)
        })
        .procedure("lobby_create", {
            <BaseProcedure>::builder().mutation(LobbyController::create)
        })
        .procedure("lobby_subscribe", {
            <BaseProcedure>::builder().subscription(LobbyController::subscribe)
        })
}
