use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::{postgres::PgArguments, Postgres};
use std::{
    any::Any,
    fmt::{self, Debug, Display},
    sync::Arc,
};

use crate::{
    error::AppResult,
    repository::{CursorDirection, Repository},
};

#[derive(Type, Clone, Debug, Serialize, Deserialize, Default)]
pub struct PageInfo {
    pub has_next_page: bool,
    pub has_prev_page: bool,
    pub start_cursor: Option<String>,
    pub end_cursor: Option<String>,
    pub total_count: i32,
}

pub trait Model<T> {
    fn id(&self) -> String;
    fn to_node(&self) -> T;
}

pub trait Node {
    fn id(&self) -> String;
}

#[derive(Type, Clone, Serialize, Deserialize, Debug)]
pub struct Edge<T: Node> {
    pub cursor: String,
    pub node: T,
}

#[derive(Type, Serialize, Deserialize, Debug, Clone, Default)]
pub struct ListResult<T: Node, M> {
    pub page_info: PageInfo,
    pub edges: Vec<Edge<T>>,
    pub meta: M,
}

#[derive(Type, Deserialize, Serialize, Debug)]
pub struct PaginationArgs {
    pub before: Option<String>,
    pub after: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

pub trait Cursor:
    Serialize + for<'de> Deserialize<'de> + Display + Clone + PartialEq + Debug
{
    type CursorType: Cursor;
    fn sort_key(&self) -> String;
    fn id(&self) -> String;
    fn decode(encoded: &str) -> Option<Self::CursorType>;
    fn encode(cursor: &Self::CursorType) -> String;
}

pub trait WithPagination {
    type Meta: Default;
    type CursorType: Cursor;
    fn pagination(&self) -> PaginationArgs;
    fn get_meta(&self) -> Self::Meta;
    fn to_cursor(&self, id: String) -> Self::CursorType;
}

pub async fn connection_from_repository<T, U, Pagination, R, M, CursorType>(
    args: &Pagination,
    repository: Arc<R>,
) -> AppResult<ListResult<U, M>>
where
    T: Model<U>,
    U: Node,
    CursorType: Cursor<CursorType = CursorType> + Send,
    R: Repository<T, Pagination> + Sync,
    Pagination: WithPagination<CursorType = CursorType, Meta = M>,
    M: Default,
{
    let pagination_args = args.pagination();
    let mut cursor_details: Option<(CursorDirection, CursorType)> = None;

    if let Some(cursor) = pagination_args.after {
        cursor_details = Some((
            CursorDirection::After,
            Pagination::CursorType::decode(&cursor).unwrap(),
        ));
    }
    if let Some(ref cursor) = pagination_args.before {
        cursor_details = Some((
            CursorDirection::Before,
            Pagination::CursorType::decode(&cursor).unwrap(),
        ));
    }

    let take = pagination_args.first.unwrap_or(25);

    let mut entities = repository.find(cursor_details, take + 1, &args).await?;

    let has_next_page = entities.len() as i32 > take;

    if has_next_page {
        entities.pop();
    }

    let edges: Vec<Edge<U>> = entities
        .into_iter()
        .map(|node| {
            let cursor = args.to_cursor(node.id());
            let node = node.to_node();

            Edge {
                cursor: Pagination::CursorType::encode(&cursor),
                node,
            }
        })
        .collect();

    let has_prev_page = pagination_args.before.is_some();

    let page_info = PageInfo {
        start_cursor: edges.first().map(|e| e.cursor.clone()),
        end_cursor: edges.last().map(|e| e.cursor.clone()),
        has_prev_page,
        has_next_page,
        total_count: repository.count(&args).await? as i32,
    };

    let meta = args.get_meta();

    Ok(ListResult {
        page_info,
        edges,
        meta,
    })
}
