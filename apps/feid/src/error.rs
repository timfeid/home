use serde::{Deserialize, Serialize};
use specta::Type;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Type, Serialize, Deserialize)]
pub enum AppError {
    InternalServerError(String),
    BadRequest(String),
    Unauthorized,
}
