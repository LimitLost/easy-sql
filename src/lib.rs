mod database_structs;
mod easy_executor;
mod traits;

mod drivers;
pub use drivers::*;

pub use {database_structs::*, easy_executor::*, sql_macros::*, sqlx::Row as SqlxRow, traits::*};

use {easy_macros::always_context, serde::de::DeserializeOwned};

#[cfg(test)]
mod tests;

#[always_context]
pub fn from_binary<T: DeserializeOwned>(slice: &[u8]) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(slice, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
pub fn from_binary_vec<T: DeserializeOwned>(v: Vec<u8>) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(&v, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
pub fn to_binary<T: serde::Serialize>(value: T) -> anyhow::Result<Vec<u8>> {
    #[no_context]
    let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;

    Ok(result)
}
pub mod macro_support;
