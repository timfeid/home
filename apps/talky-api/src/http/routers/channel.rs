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
    Router::<Ctx>::new().procedure("channel_list_in", {
        <BaseProcedure>::builder().query(|ctx, token: String| ChannelController::new(ctx).list_in())
    })
}
