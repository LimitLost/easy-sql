use easy_macros::always_context;
use sqlx::Executor;

use crate::Driver;

use super::DriverArguments;

#[always_context]
/// Conversion helper used by [`Output`].
///
/// Driver integrations provide implementations for their row types.
pub trait ToConvert<D: Driver> {
    async fn get<'a>(
        exec: impl Executor<'_, Database = D::InternalDriver>,
        query: sqlx::query::Query<'a, D::InternalDriver, DriverArguments<'a, D>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}

#[always_context]
/// Output mapping for query results.
///
/// Prefer implementing this trait via the [`Output`](crate::Output) derive macro; manual
/// implementations may need updates across releases.
pub trait Output<Table, D: Driver>: Sized {
    type DataToConvert: ToConvert<D>;
    type UsedForChecks;

    fn select(current_query: &mut String);
    fn convert(data: Self::DataToConvert) -> anyhow::Result<Self>;
}
