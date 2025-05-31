#![doc = include_str!("../README.md")]

#[cfg(feature = "not_build")]
mod database_structs;
#[cfg(feature = "not_build")]
mod easy_executor;
#[cfg(feature = "not_build")]
pub mod never;
#[cfg(feature = "not_build")]
mod sql_query;
#[cfg(feature = "not_build")]
mod traits;

#[cfg(feature = "not_build")]
pub use {
    database_structs::*, easy_executor::*, sql_compilation_data::SqlType, sql_macros::*,
    sql_query::*, sqlx::Row as SqlxRow, traits::*,
};

#[cfg(feature = "not_build")]
use {easy_macros::macros::always_context, serde::de::DeserializeOwned};

#[cfg(feature = "not_build")]
type Db = sqlx::Sqlite;
#[cfg(feature = "not_build")]
type ConnectionInternal = sqlx::pool::PoolConnection<Db>;
#[cfg(feature = "not_build")]
pub type Row = sqlx::sqlite::SqliteRow;
#[cfg(feature = "not_build")]
pub(crate) type RawConnection = sqlx::SqliteConnection;

#[cfg(test)]
#[cfg(feature = "not_build")]
use lazy_static::lazy_static;

//Used by SqlTable derive macro (default attribute)
// pub use lazy_static::lazy_static;

#[cfg(feature = "build")]
pub use sql_build::*;

#[cfg(test)]
#[cfg(feature = "not_build")]
mod tests;

#[always_context]
#[cfg(feature = "not_build")]
pub fn from_binary<T: DeserializeOwned>(slice: &[u8]) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(slice, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
#[cfg(feature = "not_build")]
pub fn from_binary_vec<T: DeserializeOwned>(v: Vec<u8>) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(&v, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
#[cfg(feature = "not_build")]
pub fn to_binary<T: serde::Serialize>(value: T) -> anyhow::Result<Vec<u8>> {
    #[no_context]
    let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;

    Ok(result)
}
