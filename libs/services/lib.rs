pub mod channel;
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
