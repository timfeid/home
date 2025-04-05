use std::{marker::PhantomData, path::PathBuf, sync::Arc};

use authentication::create_authentication_router;
use channel::create_channel_router;
use niche::create_niche_router;
use rspc::{Procedure, ProcedureBuilder, ResolverInput, ResolverOutput};

use crate::error::AppError;

use super::context::Ctx;

use std::time::{Instant, SystemTime};

use rspc::{middleware::Middleware, Router};
use serde::{Deserialize, Serialize};
use specta::Type;

mod authentication;
mod channel;
mod niche;

impl rspc::Error for AppError {
    fn into_procedure_error(self) -> rspc::ProcedureError {
        println!("{:?}", self);
        rspc::ResolverError::new(format!("{:?}", self), None::<std::io::Error>).into()
    }
}

pub struct BaseProcedure<TErr = AppError>(PhantomData<TErr>);
impl<TErr> BaseProcedure<TErr> {
    pub fn builder<TInput, TResult>(
    ) -> ProcedureBuilder<TErr, Ctx, Ctx, TInput, TInput, TResult, TResult>
    where
        TErr: rspc::Error,
        TInput: ResolverInput,
        TResult: ResolverOutput<TErr>,
    {
        Procedure::builder() // You add default middleware here
    }
}

#[derive(Type)]
struct SerialisationError;
impl Serialize for SerialisationError {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::Error;
        Err(S::Error::custom("lol"))
    }
}

pub fn mount() -> Router<Ctx> {
    Router::new()
        .merge(create_authentication_router())
        .merge(create_channel_router())
        .merge(create_niche_router())
}

pub fn timing_middleware<TError, TCtx, TInput, TResult>(
) -> Middleware<TError, TCtx, TInput, (TResult, String), TCtx, TInput, TResult>
where
    TError: Send + 'static,
    TCtx: Send + 'static,
    TInput: Send + 'static,
    TResult: Send + Sync + 'static,
{
    Middleware::new(move |ctx: TCtx, input: TInput, next| async move {
        let instant = Instant::now();
        let result = next.exec(ctx, input).await?;
        Ok((result, format!("{:?}", instant.elapsed())))
    })
}
