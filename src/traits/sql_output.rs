use anyhow::Context;
use async_trait::async_trait;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::{Executor, Row as SqlxRow};

use crate::{Db, QueryData, Row, Sql};

#[always_context]
#[async_trait]
pub trait ToConvert {
    async fn get<'a>(
        exec: impl Executor<'a, Database = Db>,
        query: sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}
#[always_context]
pub trait ToConvertSingle: ToConvert + sqlx::Row {}

#[always_context]
#[async_trait]
impl ToConvert for Row {
    async fn get<'a>(
        exec: impl Executor<'a, Database = Db>,
        query: sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>,
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
impl ToConvertSingle for Row {}

#[always_context]
#[async_trait]
impl ToConvert for () {
    async fn get<'a>(
        exec: impl Executor<'a, Database = Db>,
        query: sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>,
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
#[async_trait]
impl ToConvert for Vec<Row> {
    async fn get<'a>(
        exec: impl Executor<'a, Database = Db>,
        query: sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>,
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
pub trait SqlOutput<Table, DataToConvert: ToConvert>: Sized {
    fn sql_to_query<'a>(sql: &'a Sql<'a>) -> anyhow::Result<QueryData<'a>>;
    fn convert(data: DataToConvert) -> anyhow::Result<Self>;
}

#[always_context]
impl<Table, T> SqlOutput<Table, Vec<Row>> for Vec<T>
where
    T: SqlOutput<Table, Row>,
{
    fn sql_to_query<'a>(sql: &'a Sql<'a>) -> anyhow::Result<QueryData<'a>> {
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
impl<Table> SqlOutput<Table, ()> for () {
    fn sql_to_query<'a>(sql: &'a Sql<'a>) -> anyhow::Result<QueryData<'a>> {
        sql.query()
    }
    fn convert(_data: ()) -> anyhow::Result<Self> {
        Ok(())
    }
}

#[always_context]
impl<Table> SqlOutput<Table, Row> for bool {
    fn sql_to_query<'a>(sql: &'a Sql<'a>) -> anyhow::Result<QueryData<'a>> {
        sql.query()
    }
    fn convert(data: Row) -> anyhow::Result<Self> {
        Ok(data.get(0))
    }
}
