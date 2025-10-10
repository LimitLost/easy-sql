use easy_macros::macros::always_context;
use sqlx::Executor;

use crate::{Driver, DriverArguments, QueryData, Sql};

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
pub trait SqlOutput<Table, D: Driver, DataToConvert: ToConvert<D>>: Sized {
    fn sql_to_query<'a>(sql: &'a Sql<'a, D>) -> anyhow::Result<QueryData<'a, D>>;
    fn convert(data: DataToConvert) -> anyhow::Result<Self>;
}
