use easy_macros::macros::always_context;
use sqlx::Executor;

use crate::{Driver, DriverArguments, QueryBuilder, QueryData, Sql};

#[always_context]
pub trait ToConvert<D: Driver> {
    async fn get<'a>(
        exec: impl Executor<'a, Database = D::InternalDriver>,
        query: sqlx::query::Query<'a, D::InternalDriver, DriverArguments<'a, D>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}
#[always_context]
pub trait ToConvertSingle<D: Driver>: ToConvert<D> + sqlx::Row {}

#[always_context]
pub trait SqlOutput<Table, D: Driver>: Sized {
    type DataToConvert: ToConvert<D>;
    fn sql_to_query<'a>(sql: Sql, builder: QueryBuilder<'a, D>)
    -> anyhow::Result<QueryData<'a, D>>;

    fn select_sqlx(current_query: &mut String);
    fn convert(data: Self::DataToConvert) -> anyhow::Result<Self>;
}
