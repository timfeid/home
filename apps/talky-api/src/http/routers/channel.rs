use std::sync::Arc;

use rspc::Router;
use talky_services::{
    channel::service::ListChannelArgs, message::service::ListMessageArgs,
    user::service::ListUserArgs,
};

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
        // .procedure("channel_list", {
        //     <BaseProcedure>::builder()
        //         .query(|ctx, args: ListChannelArgs| ChannelController::new(ctx).list(args))
        // })
        .procedure("channel_find_by_slug", {
            <BaseProcedure>::builder()
                .query(|ctx, slug: String| ChannelController::new(ctx).find_by_slug(slug))
        })
        .procedure("channel_list_users", {
            <BaseProcedure>::builder()
                .query(|ctx, args: ListUserArgs| ChannelController::new(ctx).list_users(args))
        })
        .procedure("channel_messages", {
            <BaseProcedure>::builder()
                .query(|ctx, args: ListMessageArgs| ChannelController::new(ctx).list_messages(args))
        })
    // .procedure("channel_create_temporary", {
    //     <BaseProcedure>::builder()
    //         .query(|ctx, args: CreateChannelArgs| ChannelController::new(ctx).create(args))
    // })
    // .procedure("channel_list_in", {
    //     <BaseProcedure>::builder()
    //         .query(|ctx, token: String| ChannelController::new(ctx).list_in())
    // })
}
