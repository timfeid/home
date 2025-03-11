use std::{
    collections::HashMap,
    sync::Arc,
    thread::{sleep, Thread},
    time::Duration,
};

use async_stream::stream;
use futures::{pin_mut, Stream};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::{
    sync::{Mutex, MutexGuard},
    time::interval,
};
use tokio_stream::StreamExt;

use crate::{
    error::{AppError, AppResult},
    http::context::Ctx,
    lobby::{
        lobby::{Lobby, LobbyChat, LobbyData},
        manager::LobbyManager,
    },
    services::jwt::{Claims, JwtService},
};

pub struct LobbyController {}

#[derive(Type, Serialize, Deserialize)]
pub struct LobbySubscribeArgs {
    join_code: String,
    access_token: String,
}

impl LobbyController {
    pub async fn create(ctx: Ctx, _: ()) -> AppResult<LobbyData> {
        let user = ctx.required_user()?;
        let code = ctx.lobby_manager.create_lobby(user).await?;
        let lobby = ctx
            .lobby_manager
            .get_lobby(&code)
            .await
            .map_err(|x| AppError::BadRequest("No such lobby".to_string()))?;
        let data = lobby.lock().await.data.clone();

        let lobby_manager = ctx.lobby_manager.clone();
        lobby_manager.notify_lobby(&code).await.ok();

        Ok(data)
    }

    pub async fn join(ctx: Ctx, join_code: String) -> AppResult<()> {
        let user = ctx.required_user()?;
        ctx.lobby_manager
            .join_lobby(&join_code, user)
            .await
            .ok_or(AppError::BadRequest("Bad lobby id".to_string()))?;

        Ok(())
    }

    pub async fn subscribe(
        ctx: Ctx,
        subscribe_args: LobbySubscribeArgs,
    ) -> AppResult<impl Stream<Item = AppResult<LobbyData>> + Send + 'static> {
        let manager = Arc::clone(&ctx.lobby_manager);
        let user_claims = JwtService::decode(&subscribe_args.access_token)
            .unwrap()
            .claims;

        Ok(async_stream::stream! {
            match manager.subscribe_to_lobby_updates(subscribe_args.join_code, user_claims).await {
                Ok(mut post_stream) => {
                    println!("Subscribed to lobby updates");
                    pin_mut!(post_stream);

                    while let Some(item) = post_stream.next().await {
                        yield Ok(item);
                    }
                }
                Err(e) => {
                    eprintln!("Error subscribing to lobby updates: {:?}", e);
                }
            }
        })
    }
}
