use std::{marker::PhantomData, path::PathBuf, sync::Arc};

use authentication::create_authentication_router;
use lobby::create_lobby_router;
use rspc::{Procedure, ProcedureBuilder, ResolverInput, ResolverOutput};

use crate::error::AppError;

use super::context::Ctx;

use std::time::{Instant, SystemTime};
use thiserror::Error;

use rspc::{middleware::Middleware, Router};
// use rspc_cache::{cache, cache_ttl, CacheState, Memory};
// use rspc_invalidation::Invalidate;
// use rspc_zer::Zer;
use serde::{Deserialize, Serialize};
use specta::Type;
// use thiserror::Error;
// use validator::Validate;

mod authentication;
mod lobby;

// pub fn create_router() -> Arc<rspc::Router<Ctx>> {
//     let router = rspc::Router::<Ctx>::new()
//         .merge("authentication.", create_authentication_router())
//         .merge("lobby.", create_lobby_router())
//         .build()
//         .arced();

//     // prob can just be a command ? leaving it here for now
//     router
//         .export_ts(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bindings.ts"))
//         .expect("Unable to export ts bindings.");

//     router
// }

#[derive(Clone, Serialize, Deserialize, Type)]
pub struct MySession {
    name: String,
}

#[derive(Type, Serialize)]
#[serde(tag = "type")]
#[specta(export = false)]
pub enum DeserializationError {
    // Is not a map-type so invalid.
    A(String),
}

// #[derive(Debug, Error, Serialize, Type)]
// #[serde(tag = "type", content = "error")]
// pub enum Error {
//     #[error("you made a mistake: {0}")]
//     Mistake(String),
//     // #[error("validation: {0}")]
//     // Validator(#[from] rspc_validator::RspcValidatorError),
//     // #[error("authorization: {0}")]
//     // Authorization(#[from] rspc_zer::UnauthorizedError), // TODO: This ends up being cringe: `{"type":"Authorization","error":"Unauthorized"}`
//     #[error("internal error: {0}")]
//     #[serde(skip)]
//     InternalError(#[from] anyhow::Error),
// }

impl rspc::Error for AppError {
    fn into_procedure_error(self) -> rspc::ProcedureError {
        // rspc::ResolverError::new(self.to_string(), Some(self)) // TODO: Typesafe way to achieve this
        rspc::ResolverError::new(
            self,
            None::<std::io::Error>, // TODO: `Some(self)` but `anyhow::Error` is not `Clone`
        )
        .into()
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
        .merge(create_lobby_router())
    // .procedure("sendMsg", {
    //     <BaseProcedure>::builder().query(|_, msg: String| async move {
    //         println!("Got message from frontend: {msg}");
    //         Ok(msg)
    //     })
    // })
    // .procedure("withoutBaseProcedure", {
    //     Procedure::builder::<AppError>().query(|ctx: Ctx, id: String| async move { Ok(()) })
    // })
    // .procedure("newstuff", {
    //     <BaseProcedure>::builder().query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
    // })
    // .procedure("newstuff2", {
    //     <BaseProcedure>::builder()
    //         // .with(invalidation(|ctx: Ctx, key, event| false))
    //         .with(Middleware::new(
    //             move |ctx: Ctx, input: (), next| async move {
    //                 let result = next.exec(ctx, input).await;
    //                 result
    //             },
    //         ))
    //         .query(|_, _: ()| async { Ok(env!("CARGO_PKG_VERSION")) })
    // })
    // .procedure("newstuffpanic", {
    //     <BaseProcedure>::builder().query(|_, _: ()| async move { Ok(todo!()) })
    // })

    // .procedure("fileupload", {
    //     <BaseProcedure>::builder().query(|_, _: File| async { Ok(env!("CARGO_PKG_VERSION")) })
    // })
}

// .with(Invalidator::mw(|ctx, input, event| {
//     event == InvalidateEvent::InvalidateKey("abc".into())
// }))
// .with(Invalidator::mw_with_result(|ctx, input, result, event| {
//     event == InvalidateEvent::InvalidateKey("abc".into())
// }))

#[derive(Debug, Clone, Serialize, Type, PartialEq, Eq)]
pub enum InvalidateEvent {
    Post { id: String },
    InvalidateKey(String),
}

// TODO: Debug, etc
pub struct File<T = ()>(T);

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
