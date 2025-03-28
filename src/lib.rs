mod database_structs;
pub use database_structs::*;
mod easy_executor;
pub use easy_executor::*;
pub mod never;
mod sql_query;
pub use sql_query::*;
mod traits;
pub use traits::*;

type Db = sqlx::Sqlite;
type ConnectionInternal = sqlx::pool::PoolConnection<Db>;
pub type Row = sqlx::sqlite::SqliteRow;

pub use sqlx::Row as SqlxRow;

pub use async_trait::async_trait;

// #[cfg(test)]
mod tests;
