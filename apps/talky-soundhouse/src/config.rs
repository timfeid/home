use crate::error::{AppError, AppResult};
use std::env;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub server_addr: SocketAddr,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let server_addr_str: String =
            env::var("SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
        let server_addr = server_addr_str
            .parse::<SocketAddr>()
            .map_err(|e| AppError::Config(env::VarError::NotPresent))?;

        let database_url =
            env::var("DATABASE_URL").map_err(|e| AppError::Config(env::VarError::NotPresent))?;

        Ok(Config {
            server_addr,
            database_url,
        })
    }
}
