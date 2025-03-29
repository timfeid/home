use std::sync::Arc;

use axum::http::request::Parts;
use sqlx::{Pool, Postgres};
use talky_auth::{Claims, JwtService};

use crate::{
    error::{AppError, AppResult},
    lobby::manager::LobbyManager,
};

#[derive(Debug)]
pub struct Ctx {
    pub pool: Arc<Pool<Postgres>>,
    user: Option<Claims>,
    pub lobby_manager: Arc<LobbyManager>,
}

impl Ctx {
    pub fn new(pool: Arc<Pool<Postgres>>, parts: Parts, lobby_manager: Arc<LobbyManager>) -> Ctx {
        let user = match parts.headers.get("Authorization") {
            Some(header_value) => {
                let token_str = header_value.to_str().unwrap_or_default();
                let token = if token_str.to_lowercase().starts_with("bearer ") {
                    &token_str[7..]
                } else {
                    token_str
                };

                JwtService::decode(token).map(|r| r.claims).ok()
            }
            None => None,
        };

        Ctx {
            pool,
            user,
            lobby_manager,
        }
    }

    pub fn required_user(self: &Ctx) -> AppResult<&Claims> {
        // println!("{:?}", self);
        if self.user.is_none() {
            return Err(AppError::Unauthorized);
        }
        // Err(AppError::Unauthorized)
        Ok(self.user.as_ref().unwrap())
    }
}
