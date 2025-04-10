pub mod category;
pub mod channel;
pub mod error;
pub mod lobby;
pub mod message;
pub mod niche;
pub mod pagination;
mod repository;
pub mod user;

use std::sync::Arc;

use sqlx::{Pool, Postgres};

pub type DatabasePool = Arc<Pool<Postgres>>;

fn main() {
    println!("Hello, world!");
}
