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
            <BaseProcedure>::builder()
                .query(|ctx, token: String| AuthenticationController::new(ctx).refresh_token(token))
        })
        .procedure("auth_login", {
            <BaseProcedure>::builder()
                .mutation(|ctx, args: LoginArgs| AuthenticationController::new(ctx).login(args))
        })
}
