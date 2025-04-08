use crate::{error::AppResult, pagination::Cursor};
use async_trait::async_trait;

#[derive(PartialEq, Clone, Copy)]
pub enum CursorDirection {
    Before,
    After,
}

pub trait Repository<T, Y> {
    async fn count(&self, args: &Y) -> AppResult<i32>;
    async fn find(
        &self,
        after: Option<(CursorDirection, impl Cursor + Send)>,
        take: i32,
        args: &Y,
    ) -> AppResult<Vec<T>>;
}
