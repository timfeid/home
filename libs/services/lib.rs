pub mod channel;
pub mod pagination;
mod repository;

use std::sync::Arc;

use sqlx::{Pool, Postgres};

pub type DatabasePool = Arc<Pool<Postgres>>;

fn main() {
    println!("Hello, world!");
}
