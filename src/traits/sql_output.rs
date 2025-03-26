use std::any::Any;

use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};

use crate::sql_query::{Column, RequestedColumn};

#[always_context]
pub trait ToConvert {}
#[always_context]
pub trait ToConvertSingle: ToConvert + sqlx::Row {}

#[always_context]
impl ToConvert for sqlx::sqlite::SqliteRow {}
#[always_context]
impl ToConvertSingle for sqlx::sqlite::SqliteRow {}

#[always_context]
impl<T: ToConvert> ToConvert for Vec<T> {}

#[always_context]
pub trait SqlOutput<Table, DataToConvert: ToConvert>: Sized {
    fn requested_columns() -> Vec<RequestedColumn>;
    fn convert(data: DataToConvert) -> anyhow::Result<Self>;
}

#[always_context]
impl<Table, T, Y: ToConvertSingle> SqlOutput<Table, Vec<Y>> for Vec<T>
where
    T: SqlOutput<Table, Y>,
{
    fn requested_columns() -> Vec<RequestedColumn> {
        T::requested_columns()
    }
    fn convert(data: Vec<Y>) -> anyhow::Result<Self> {
        let mut result = Vec::new();
        for r in data.into_iter() {
            #[no_context_inputs]
            result.push(T::convert(r)?);
        }
        Ok(result)
    }
}
