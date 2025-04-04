use bcrypt::verify;
use sqlx::{Pool, Postgres};

use rspc::Router;
use serde::{Deserialize, Serialize};
use specta::Type;
use talky_auth::JwtService;
use talky_data::models::user::User;

use crate::{
    error::{AppError, AppResult},
    http::context::Ctx,
};

#[derive(Type, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl AuthResponse {
    async fn new(pool: &Pool<Postgres>, user: User) -> AppResult<AuthResponse> {
        let jti = user
            .create_refresh_token(pool)
            .await
            .map_err(|m| AppError::InternalServerError(m.to_string()))?;

        Ok(AuthResponse {
            access_token: JwtService::create_for_user(&user, None)
                .map_err(|m| AppError::InternalServerError(m.to_string()))?,
            refresh_token: JwtService::create_for_user(&user, Some(jti))
                .map_err(|m| AppError::InternalServerError(m.to_string()))?,
        })
    }
}

#[derive(Type, Deserialize)]
pub struct LoginArgs {
    username: String,
    password: String,
}

pub struct AuthenticationController {
    ctx: Ctx,
}

impl AuthenticationController {
    pub async fn login(self, args: LoginArgs) -> AppResult<AuthResponse> {
        if let Ok(user) = User::find(&self.ctx.pool, &args.username).await {
            if user.verify_password(&args.password) {
                return AuthResponse::new(&self.ctx.pool, user).await;
            }
        }

        Err(AppError::BadRequest(
            "Invalid username or password".to_string(),
        ))
    }

    pub async fn refresh_token(self, token: String) -> AppResult<AuthResponse> {
        let details = JwtService::decode(&token)
            .map_err(|_| AppError::BadRequest("Invalid token".to_owned()))?;

        if let Ok(user) = User::find_by_refresh_token(
            &self.ctx.pool,
            (&details.claims.sub, &details.claims.jti.unwrap_or_default()),
        )
        .await
        {
            return AuthResponse::new(&self.ctx.pool, user).await;
        }

        Err(AppError::BadRequest("Invalid token".to_string()))
    }

    pub(crate) fn new(ctx: Ctx) -> Self {
        Self { ctx }
    }
}
