#![doc = include_str!("../README.md")]

mod database_structs;
pub use database_structs::*;
mod easy_executor;
pub use easy_executor::*;
pub mod never;
mod sql_query;
use easy_macros::macros::always_context;
use serde::de::DeserializeOwned;
pub use sql_query::*;
mod traits;
pub use traits::*;

type Db = sqlx::Sqlite;
type ConnectionInternal = sqlx::pool::PoolConnection<Db>;
pub type Row = sqlx::sqlite::SqliteRow;

pub(crate) type RawConnection = sqlx::SqliteConnection;

pub use sqlx::Row as SqlxRow;

pub use sql_compilation_data::SqlType;

pub use async_trait::async_trait;

pub use sql_macros::*;

#[cfg(test)]
mod tests;

#[always_context]
pub fn from_binary<T: DeserializeOwned>(slice: &[u8]) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(slice, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
pub fn to_binary<T: serde::Serialize>(value: T) -> anyhow::Result<Vec<u8>> {
    #[no_context]
    let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;

    Ok(result)
}
