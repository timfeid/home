use std::sync::Arc;

use rspc::Router;

use crate::http::{
    context::Ctx,
    controllers::{
        authentication::{AuthenticationController, LoginArgs},
        channel::ChannelController,
    },
};

use super::BaseProcedure;

pub fn create_channel_router() -> Router<Ctx> {
    Router::<Ctx>::new()
        .procedure("channel_find_by_slug", {
            <BaseProcedure>::builder()
                .query(|ctx, slug: String| ChannelController::new(ctx).find_by_slug(slug))
        })
        .procedure("channel_list_users", {
            <BaseProcedure>::builder()
                .query(|ctx, id: String| ChannelController::new(ctx).list_users(id))
        })
        .procedure("channel_list_in", {
            <BaseProcedure>::builder()
                .query(|ctx, token: String| ChannelController::new(ctx).list_in())
        })
}
