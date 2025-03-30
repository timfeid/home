use thiserror::Error;
use warp::ws::Message;
use warp::Error as WarpError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] std::env::VarError),

    #[error("JWT authentication error: {0}")]
    JwtAuth(String),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WarpError),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Client sent invalid message format")]
    InvalidMessageFormat,

    #[error("Missing or invalid field in message: {0}")]
    MissingField(String),

    #[error("Client send error")]
    ClientSendError,

    #[error("Client disconnected unexpectedly")]
    ClientDisconnected,

    #[error("Initialization message error: {0}")]
    InitializationError(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn to_ws_close_message(&self) -> Message {
        let (code, reason) = match self {
            AppError::JwtAuth(_) => (1008, "Authentication failed"),
            AppError::InvalidMessageFormat | AppError::MissingField(_) => {
                (1007, "Invalid message format")
            }
            AppError::InitializationError(_) => (1002, "Protocol error"),
            _ => (1011, "Internal server error"),
        };
        Message::close_with(code as u16, reason)
    }
}
