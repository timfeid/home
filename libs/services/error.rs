use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServicesError {
    #[error("Configuration error: {0}")]
    Config(#[from] std::env::VarError),

    #[error("SQL Error: {0}")]
    SQLError(String),
}

pub type AppResult<T> = Result<T, ServicesError>;

impl From<sqlx::Error> for ServicesError {
    fn from(value: sqlx::Error) -> Self {
        eprintln!("SQL Error: {:?}", value);
        ServicesError::SQLError("Something went wrong.".to_string())
    }
}
