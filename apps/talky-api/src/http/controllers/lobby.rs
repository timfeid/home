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
        lobby::{Lobby, LobbyChat, LobbyCommand, LobbyData},
        manager::LobbyManager,
    },
    services::jwt::{Claims, JwtService},
};

pub struct LobbyController {}

#[derive(Type, Serialize, Deserialize)]
pub struct LobbyPingArgs {
    join_code: String,
    socket_id: String,
}

#[derive(Type, Serialize, Deserialize)]
pub struct LobbySubscribeArgs {
    join_code: String,
    access_token: String,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct Member {
    name: String,
    user_id: String,
    avatar: Option<String>,
    color: Option<String>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct MemberList {
    name: String,
    members: Vec<Member>,
}

#[derive(Type, Deserialize, Serialize, Debug, Clone)]
pub struct LobbyResponse {
    join_code: String,
    member_list: Vec<MemberList>,
}
impl LobbyResponse {
    fn from_join_code(code: &str) -> LobbyResponse {
        LobbyResponse {
            join_code: code.to_string(),
            member_list: vec![MemberList {
                name: "hello".to_string(),
                members: vec![Member {
                    name: "dazed".to_string(),
                    user_id: "tim".to_string(),
                    avatar: None,
                    color: None,
                }],
            }],
        }
    }
}

impl LobbyController {
    pub async fn create(ctx: Ctx, _: ()) -> AppResult<LobbyResponse> {
        let user = ctx.required_user()?;
        let code = ctx.lobby_manager.create_lobby(user).await?;

        Ok(LobbyResponse::from_join_code(&code))
    }

    pub async fn pong(ctx: Ctx, args: LobbyPingArgs) -> AppResult<()> {
        let user = ctx.required_user()?;

        ctx.lobby_manager
            .pong(&args.join_code, &args.socket_id, user.clone())
            .await
    }

    pub async fn join(ctx: Ctx, join_code: String) -> AppResult<LobbyResponse> {
        let user = ctx.required_user()?;
        ctx.lobby_manager
            .join_lobby(&join_code, user)
            .await
            .ok_or(AppError::BadRequest("Bad lobby id".to_string()))?;

        Ok(LobbyResponse::from_join_code(&join_code))
    }

    pub async fn subscribe(
        ctx: Ctx,
        subscribe_args: LobbySubscribeArgs,
    ) -> AppResult<impl Stream<Item = AppResult<LobbyCommand>> + Send + 'static> {
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
                    yield Err(e)
                }
            }
        })
    }
}
