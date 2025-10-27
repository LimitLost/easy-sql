use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::{Executor, Row as SqlxRow};

/// Current Driver
type CDriver = super::Sqlite;

use crate::{
    DriverArguments, InternalDriver, Output, QueryBuilder, QueryData, Sql, ToConvert,
    ToConvertSingle,
};

type Row = sqlx::sqlite::SqliteRow;

#[always_context]
impl ToConvert<CDriver> for Row {
    async fn get<'a>(
        exec: impl Executor<'a, Database = InternalDriver<CDriver>>,
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
impl<T, Table> Output<Table, CDriver> for Vec<T>
where
    T: Output<Table, CDriver, DataToConvert = Row>,
{
    type DataToConvert = Vec<Row>;
    fn sql_to_query<'a>(
        sql: Sql,
        builder: QueryBuilder<'a, CDriver>,
    ) -> anyhow::Result<QueryData<'a, CDriver>> {
        T::sql_to_query(sql, builder)
    }

    fn select_sqlx(current_query: &mut String) {
        T::select_sqlx(current_query);
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
    fn sql_to_query<'a>(
        sql: Sql,
        builder: QueryBuilder<'a, CDriver>,
    ) -> anyhow::Result<QueryData<'a, CDriver>> {
        T::sql_to_query(sql, builder)
    }

    fn select_sqlx(current_query: &mut String) {
        T::select_sqlx(current_query);
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
    fn sql_to_query<'a>(
        sql: Sql,
        builder: QueryBuilder<'a, CDriver>,
    ) -> anyhow::Result<QueryData<'a, CDriver>> {
        sql.query(builder)
    }

    fn select_sqlx(current_query: &mut String) {
        current_query.push('1');
    }

    fn convert(_data: ()) -> anyhow::Result<Self> {
        Ok(())
    }
}

#[always_context]
impl<Table> Output<Table, CDriver> for bool {
    type DataToConvert = Row;
    fn sql_to_query<'a>(
        sql: Sql,
        builder: QueryBuilder<'a, CDriver>,
    ) -> anyhow::Result<QueryData<'a, CDriver>> {
        sql.query(builder)
    }

    fn select_sqlx(_current_query: &mut String) {
        panic!(
            "Usage of `bool` type as output of sql_full! macro should be dissallowed by the macro itself. Are you using type to circumvent security checks? ;) If not please report this bug to easy-sql repository."
        );
    }

    fn convert(data: Row) -> anyhow::Result<Self> {
        Ok(data.get(0))
    }
}
