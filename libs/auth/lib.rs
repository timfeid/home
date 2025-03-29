use std::ops::Add;

use jsonwebtoken::{
    decode, encode, get_current_timestamp, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use talky_data::models::user::User;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub jti: Option<String>,
    pub exp: u64,
}

pub struct JwtService {}

impl JwtService {
    pub fn decode(token: &str) -> anyhow::Result<jsonwebtoken::TokenData<Claims>> {
        return Ok(decode(
            token,
            &DecodingKey::from_rsa_pem(include_bytes!("./jwt.public.pem"))?,
            &Validation::new(Algorithm::RS256),
        )?);
    }

    pub fn create_for_user(user: &User, jti: Option<String>) -> anyhow::Result<String> {
        let is_access_token = &jti.is_none();
        let claims = Claims {
            jti,
            sub: user.get_id().to_string(),
            exp: get_current_timestamp().add(match is_access_token {
                &true => 3600,
                &false => 604800,
            }),
        };

        let response = encode(
            &Header::new(Algorithm::RS256),
            &claims,
            &EncodingKey::from_rsa_pem(include_bytes!("./jwt.private.pem"))?,
        );

        Ok(response?)
    }
}
