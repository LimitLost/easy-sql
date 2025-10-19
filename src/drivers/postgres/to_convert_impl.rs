use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::{Executor, Row as SqlxRow};

/// Current Driver
type CDriver = super::Postgres;

use crate::{
    DriverArguments, InternalDriver, QueryData, Sql, SqlOutput, ToConvert, ToConvertSingle,
};

type Row = sqlx::postgres::PgRow;

#[always_context]
impl ToConvert<CDriver> for Row {
    async fn get<'a, 'b>(
        exec: impl Executor<'a, Database = InternalDriver<CDriver>>,
        query: sqlx::query::Query<'b, InternalDriver<CDriver>, DriverArguments<'b, CDriver>>,
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
        exec: impl Executor<'a, Database = InternalDriver<CDriver>>,
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
        exec: impl Executor<'a, Database = InternalDriver<CDriver>>,
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
        exec: impl Executor<'a, Database = InternalDriver<CDriver>>,
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
impl<T, Table> SqlOutput<Table, CDriver, Vec<Row>> for Vec<T>
where
    T: SqlOutput<Table, CDriver, Row>,
{
    fn sql_to_query<'a>(sql: &'a Sql<'a, CDriver>) -> anyhow::Result<QueryData<'a, CDriver>> {
        T::sql_to_query(sql)
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
impl<T, Table> SqlOutput<Table, CDriver, Option<Row>> for Option<T>
where
    T: SqlOutput<Table, CDriver, Row>,
{
    fn sql_to_query<'a>(sql: &'a Sql<'a, CDriver>) -> anyhow::Result<QueryData<'a, CDriver>> {
        T::sql_to_query(sql)
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
impl<Table> SqlOutput<Table, CDriver, ()> for () {
    fn sql_to_query<'a>(sql: &'a Sql<'a, CDriver>) -> anyhow::Result<QueryData<'a, CDriver>> {
        sql.query()
    }
    fn convert(_data: ()) -> anyhow::Result<Self> {
        Ok(())
    }
}

#[always_context]
impl<Table> SqlOutput<Table, CDriver, Row> for bool {
    fn sql_to_query<'a>(sql: &'a Sql<'a, CDriver>) -> anyhow::Result<QueryData<'a, CDriver>> {
        sql.query()
    }
    fn convert(data: Row) -> anyhow::Result<Self> {
        Ok(data.get(0))
    }
}
