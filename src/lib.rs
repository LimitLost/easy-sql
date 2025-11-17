#![doc = include_str!("../README.md")]

#[cfg(feature = "not_build")]
mod database_structs;
#[cfg(feature = "not_build")]
mod easy_executor;
#[cfg(feature = "not_build")]
mod query;
#[cfg(feature = "not_build")]
mod traits;

#[cfg(feature = "not_build")]
mod drivers;
#[cfg(feature = "not_build")]
pub use drivers::*;

#[cfg(feature = "not_build")]
pub use {
    database_structs::*, easy_executor::*, query::*, sql_macros::*, sqlx::Row as SqlxRow, traits::*,
};

#[cfg(feature = "not_build")]
use {easy_macros::always_context, serde::de::DeserializeOwned};

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
#[cfg(feature = "not_build")]
pub mod macro_support;
