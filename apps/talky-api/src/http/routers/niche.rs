use std::sync::Arc;

use rspc::Router;
use talky_services::niche::service::ListNicheArgs;

use crate::http::{
    context::Ctx,
    controllers::{
        authentication::{AuthenticationController, LoginArgs},
        niche::NicheController,
    },
};

use super::BaseProcedure;

pub fn create_niche_router() -> Router<Ctx> {
    Router::<Ctx>::new()
        .procedure("niche_find_by_slug", {
            <BaseProcedure>::builder()
                .query(|ctx, slug: String| NicheController::new(ctx).find_by_slug(slug))
        })
        .procedure("niche_list", {
            <BaseProcedure>::builder()
                .query(|ctx, args: ListNicheArgs| NicheController::new(ctx).list(args))
        })
}
