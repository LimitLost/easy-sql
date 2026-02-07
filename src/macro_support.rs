//! Reexported items and support functions used by procedural macros

pub use anyhow::{Context, Error, Result};
use easy_macros::always_context;
pub use easy_macros::context;
pub use futures_core::Stream;
use serde::de::DeserializeOwned;
use sqlx::IntoArguments;
pub use sqlx::{
    Arguments, ColumnIndex, Decode, Encode, Executor, QueryBuilder, Type, TypeInfo, query::Query,
    query_with,
};

pub use crate::traits::{
    DriverArguments, DriverConnection, DriverQueryResult, DriverRow, DriverTypeInfo,
    InternalDriver, ToConvert,
};

pub use crate::markers::OutputData;

use crate::traits::{Driver, EasyExecutor, Insert, Output, Table, Update};

pub use sqlx::Row as SqlxRow;

/// Used for compiler checks, quickly creates a value of any type
///
/// Panics if called
///
pub fn never_any<T>() -> T {
    panic!(
        "This function should never be called in runtime, it's used to quickly create value for type in compiler checks"
    );
}

#[inline(always)]
///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
pub fn args_for_driver<'a, D: Driver>(
    _exec: &impl crate::EasyExecutor<D>,
) -> DriverArguments<'a, D> {
    DriverArguments::<D>::default()
}
///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
#[inline(always)]
pub fn driver_identifier_delimiter<D: Driver>(_exec: &impl crate::EasyExecutor<D>) -> &'static str {
    D::identifier_delimiter()
}
///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
pub fn driver_parameter_placeholder<D: Driver>(
    _exec: &impl crate::EasyExecutor<D>,
) -> Box<dyn Fn(usize) -> String> {
    Box::new(|index: usize| D::parameter_placeholder(index))
}

/// This function extracts Driver from connection to fetch type information for a Rust type.
#[inline(always)]
pub fn driver_type_info<T, D: Driver>(
    _exec: &impl crate::EasyExecutor<D>,
) -> <InternalDriver<D> as sqlx::Database>::TypeInfo
where
    T: Type<InternalDriver<D>>,
{
    <T as Type<InternalDriver<D>>>::type_info()
}

///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
#[inline(always)]
pub fn query_add_selected<T, O: Output<T, D>, D: Driver>(
    query: &mut String,
    _exec: &impl crate::EasyExecutor<D>,
) where
    DriverRow<D>: ToConvert<D>,
{
    O::select(query);
}

#[always_context(skip(!))]
#[inline(always)]
///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
pub fn query_insert_data<'a, Table, D: Driver, T: Insert<'a, Table, D>>(
    to_insert: T,
    args: DriverArguments<'a, D>,
    _exec: &impl crate::EasyExecutor<D>,
) -> anyhow::Result<(Vec<String>, DriverArguments<'a, D>, usize)> {
    to_insert
        .insert_values(args)
        .map(|(new_args, count)| (T::insert_columns(), new_args, count))
        .context("Insert::insert_values failed")
}

#[always_context(skip(!))]
#[inline(always)]
///This function extracts Insert type (that's the only reason why it exists instead of direct call)
///
/// Driver is already known
pub fn query_insert_data_selected_driver<'a, Table, D: Driver, T: Insert<'a, Table, D>>(
    to_insert: T,
    args: DriverArguments<'a, D>,
) -> anyhow::Result<(Vec<String>, DriverArguments<'a, D>, usize)> {
    to_insert
        .insert_values(args)
        .map(|(new_args, count)| (T::insert_columns(), new_args, count))
        .context("Insert::insert_values failed")
}

#[always_context(skip(!))]
#[inline(always)]
///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
pub fn query_update_data<'a, Table, D: Driver, T: Update<'a, Table, D>>(
    update_data: T,
    args: DriverArguments<'a, D>,
    current_query: &mut String,
    parameter_n: &mut usize,
    _exec: &impl crate::EasyExecutor<D>,
) -> anyhow::Result<DriverArguments<'a, D>> {
    update_data
        .updates(args, current_query, parameter_n)
        .context("Update::updates failed")
}
#[always_context(skip(!))]
#[inline(always)]
/// This function extracts Update type (that's the only reason why it exists instead of direct call)
///
/// Driver is already known
pub fn query_update_data_selected_driver<'a, Table, D: Driver, T: Update<'a, Table, D>>(
    update_data: T,
    args: DriverArguments<'a, D>,
    current_query: &mut String,
    parameter_n: &mut usize,
) -> anyhow::Result<DriverArguments<'a, D>> {
    update_data
        .updates(args, current_query, parameter_n)
        .context("Update::updates failed")
}

///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
#[inline(always)]
pub fn driver_related_table_name<T: Table<D>, D: Driver>(
    _exec: &impl crate::EasyExecutor<D>,
) -> &'static str {
    T::table_name()
}

///This function extracts Driver from connection (that's the only reason why it exists instead of direct call)
#[inline(always)]
pub fn driver_table_joins<T: Table<D>, D: Driver>(
    query: &mut String,
    _exec: &impl crate::EasyExecutor<D>,
) {
    T::table_joins(query);
}
/// Used by UPDATE, DELETE modes of query! and query_lazy! macros
pub async fn query_execute<'a, T, O: Output<T, D>, D: Driver>(
    exec: &mut impl EasyExecutor<D>,
    query: Query<'a, InternalDriver<D>, DriverArguments<'a, D>>,
) -> Result<O>
where
    DriverArguments<'a, D>: IntoArguments<'a, InternalDriver<D>>,
{
    let raw_data = O::DataToConvert::get(exec.executor(), query)
        .await
        .context("Output::DataToConvert::get failed")?;

    O::convert(raw_data).context("Output::convert failed")
}
/// Used by UPDATE, DELETE modes of query! and query_lazy! macros
pub async fn query_execute_no_output<'a, D: Driver>(
    exec: &mut impl EasyExecutor<D>,
    query: Query<'a, InternalDriver<D>, DriverArguments<'a, D>>,
) -> Result<DriverQueryResult<D>>
where
    DriverArguments<'a, D>: IntoArguments<'a, InternalDriver<D>>,
{
    query
        .execute(exec.executor())
        .await
        .context("QueryBuilder::build.execute failed")
}

pub async fn query_exists_execute<'a, D: Driver>(
    exec: &mut impl EasyExecutor<D>,
    query: Query<'a, InternalDriver<D>, DriverArguments<'a, D>>,
) -> Result<bool>
where
    DriverArguments<'a, D>: IntoArguments<'a, InternalDriver<D>>,
    for<'x> bool: Decode<'x, InternalDriver<D>>,
    bool: Type<InternalDriver<D>>,
    usize: ColumnIndex<DriverRow<D>>,
{
    let row = query
        .fetch_one(exec.executor())
        .await
        .context("sqlx::Query::fetch_one failed")?;
    let exists: bool =
        <DriverRow<D> as sqlx::Row>::try_get(&row, 0).context("SqlxRow::try_get failed")?;

    Ok(exists)
}

#[always_context]
/// Used by #[sql(bytes)]
pub fn from_binary<T: DeserializeOwned>(slice: &[u8]) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(slice, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
/// Used by #[sql(bytes)]
pub fn from_binary_vec<T: DeserializeOwned>(v: Vec<u8>) -> anyhow::Result<T> {
    #[no_context]
    let (result, _) = bincode::serde::decode_from_slice(&v, bincode::config::standard())?;

    Ok(result)
}

#[always_context]
/// Used by #[sql(bytes)]
pub fn to_binary<T: serde::Serialize>(value: T) -> anyhow::Result<Vec<u8>> {
    #[no_context]
    let result = bincode::serde::encode_to_vec(value, bincode::config::standard())?;

    Ok(result)
}

/// Const hash for SQL function names to use in compile-time capability checks.
///
/// Uses a simple FNV-1a 64-bit hash for stable, reproducible IDs.
pub const fn function_name_hash(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}
