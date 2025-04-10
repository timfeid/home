use std::sync::Arc;

use rspc::Router;
use talky_services::{
    category::service::{CreateCategoryArgs, ListCategoryArgs},
    message::service::ListMessageArgs,
    user::service::ListUserArgs,
};

use crate::http::{
    context::Ctx,
    controllers::{
        authentication::{AuthenticationController, LoginArgs},
        category::CategoryController,
    },
};

use super::BaseProcedure;

pub fn create_category_router() -> Router<Ctx> {
    Router::<Ctx>::new().procedure("category_list", {
        <BaseProcedure>::builder()
            .query(|ctx, args: ListCategoryArgs| CategoryController::new(ctx).list(args))
    })
}
