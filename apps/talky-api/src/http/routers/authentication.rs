use std::sync::Arc;

use rspc::Router;

use crate::http::{
    context::Ctx,
    controllers::authentication::{AuthenticationController, LoginArgs},
};

use super::BaseProcedure;

pub fn create_authentication_router() -> Router<Ctx> {
    Router::<Ctx>::new()
        .procedure("auth_refresh_token", {
            <BaseProcedure>::builder().query(AuthenticationController::refresh_token)
        })
        .procedure("auth_login", {
            <BaseProcedure>::builder().mutation(AuthenticationController::login)
        })
}
