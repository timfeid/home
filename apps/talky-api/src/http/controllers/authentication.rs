use bcrypt::verify;
use sqlx::{Pool, Postgres};

use rspc::Router;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::{
    error::{AppError, AppResult},
    http::context::Ctx,
    models::user::User,
    services::jwt::JwtService,
};

#[derive(Type, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl AuthResponse {
    async fn new(pool: &Pool<Postgres>, user: User) -> AppResult<AuthResponse> {
        let jti = user.create_refresh_token(pool).await?;

        Ok(AuthResponse {
            access_token: JwtService::create_for_user(&user, None)?,
            refresh_token: JwtService::create_for_user(&user, Some(jti))?,
        })
    }
}

#[derive(Type, Deserialize)]
pub struct LoginArgs {
    username: String,
    password: String,
}

pub struct AuthenticationController {}
impl AuthenticationController {
    pub async fn login(ctx: Ctx, args: LoginArgs) -> AppResult<AuthResponse> {
        if let Ok(user) = User::find(&ctx.pool, &args.username).await {
            if user.verify_password(&args.password) {
                return AuthResponse::new(&ctx.pool, user).await;
            }
        }

        Err(AppError::BadRequest(
            "Invalid username or password".to_string(),
        ))
    }

    pub async fn refresh_token(ctx: Ctx, token: String) -> AppResult<AuthResponse> {
        let details = JwtService::decode(&token)
            .map_err(|_| AppError::BadRequest("Invalid token".to_owned()))?;

        if let Ok(user) = User::find_by_refresh_token(
            &ctx.pool,
            (&details.claims.sub, &details.claims.jti.unwrap_or_default()),
        )
        .await
        {
            return AuthResponse::new(&ctx.pool, user).await;
        }

        Err(AppError::BadRequest("Invalid token".to_string()))
    }
}
