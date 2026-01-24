use anyhow::Context;
use easy_macros::{always_context, context};
use sqlx::Executor;

/// Current Driver
type CDriver = super::Sqlite;

use crate::{DriverArguments, InternalDriver, Output, ToConvert, ToConvertSingle};

type Row = sqlx::sqlite::SqliteRow;

#[always_context]
impl ToConvert<CDriver> for Row {
    async fn get<'a>(
        exec: impl Executor<'_, Database = InternalDriver<CDriver>>,
        query: sqlx::query::Query<'a, InternalDriver<CDriver>, DriverArguments<'a, CDriver>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        exec.fetch_one(query)
            .await
            .with_context(context!("Failed to fetch one row from SQL query"))
    }
}
#[always_context]
impl ToConvertSingle<CDriver> for Row {}

#[always_context]
impl ToConvert<CDriver> for Option<Row> {
    async fn get<'a>(
        exec: impl Executor<'_, Database = InternalDriver<CDriver>>,
        query: sqlx::query::Query<'a, InternalDriver<CDriver>, DriverArguments<'a, CDriver>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        exec.fetch_optional(query)
            .await
            .with_context(context!("Failed to fetch optional row from SQL query"))
    }
}

#[always_context]
impl ToConvert<CDriver> for () {
    async fn get<'a>(
        exec: impl Executor<'_, Database = InternalDriver<CDriver>>,
        query: sqlx::query::Query<'a, InternalDriver<CDriver>, DriverArguments<'a, CDriver>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        #[no_context_inputs]
        exec.execute(query)
            .await
            .with_context(context!("Failed to execute SQL query"))?;
        Ok(())
    }
}

#[always_context]
impl ToConvert<CDriver> for Vec<Row> {
    async fn get<'a>(
        exec: impl Executor<'_, Database = InternalDriver<CDriver>>,
        query: sqlx::query::Query<'a, InternalDriver<CDriver>, DriverArguments<'a, CDriver>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        exec.fetch_all(query)
            .await
            .with_context(context!("Failed to fetch all rows from SQL query"))
    }
}

#[always_context]
impl<T, Table> Output<Table, CDriver> for Vec<T>
where
    T: Output<Table, CDriver, DataToConvert = Row>,
{
    type DataToConvert = Vec<Row>;
    type UsedForChecks = T::UsedForChecks;

    fn select(current_query: &mut String) {
        T::select(current_query);
    }

    fn convert(data: Vec<Row>) -> anyhow::Result<Self> {
        let mut result = Vec::new();
        for r in data.into_iter() {
            #[no_context_inputs]
            result.push(T::convert(r)?);
        }
        Ok(result)
    }
}

#[always_context]
impl<T, Table> Output<Table, CDriver> for Option<T>
where
    T: Output<Table, CDriver, DataToConvert = Row>,
{
    type DataToConvert = Option<Row>;
    type UsedForChecks = T::UsedForChecks;

    fn select(current_query: &mut String) {
        T::select(current_query);
    }

    fn convert(data: Option<Row>) -> anyhow::Result<Self> {
        Ok(if let Some(data) = data {
            #[no_context_inputs]
            Some(T::convert(data)?)
        } else {
            None
        })
    }
}

#[always_context]
impl<Table> Output<Table, CDriver> for () {
    type DataToConvert = ();
    type UsedForChecks = ();

    fn select(current_query: &mut String) {
        current_query.push('1');
    }

    fn convert(_data: ()) -> anyhow::Result<Self> {
        Ok(())
    }
}
