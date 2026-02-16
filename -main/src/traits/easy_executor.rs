use easy_macros::always_context;

use crate::Driver;

use super::DriverConnection;

#[always_context]
/// Abstraction over [`sqlx::Executor`](https://docs.rs/sqlx/latest/sqlx/trait.Executor.html) used by the query macros.
///
/// Implemented for easy_sql and sqlx connections/pools; most users only need to pass a compatible
/// connection to [`query!`](crate::query) or [`query_lazy!`](crate::query_lazy).
///
/// Will contain sql query watch functions in the future (gated by a feature)
pub trait EasyExecutor<D: Driver> {
    type InternalExecutor<'b>: sqlx::Executor<'b, Database = D::InternalDriver>
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync;

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a>;
}

pub trait EasyExecutorInto<D: Driver>: EasyExecutor<D> {
    type IntoInternalExecutor<'b>: sqlx::Executor<'b, Database = D::InternalDriver>
    where
        Self: 'b;

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a;
}

#[always_context(skip(!))]
impl<D: Driver, E: EasyExecutor<D> + ?Sized> EasyExecutor<D> for &mut E {
    type InternalExecutor<'b>
        = E::InternalExecutor<'b>
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync,
    {
        (**self).query_setup(sql).await
    }

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a> {
        (**self).executor()
    }
}

impl<D: Driver, E: EasyExecutor<D> + ?Sized> EasyExecutorInto<D> for &mut E {
    type IntoInternalExecutor<'b>
        = E::InternalExecutor<'b>
    where
        Self: 'b;

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        (*self).executor()
    }
}

#[always_context]
/// Gives information about the SQL query to execute, and how to execute it with the provided executor. Used by setup structures.
pub trait SetupSql<D: Driver> {
    type Output;

    async fn query(self, exec: &mut impl EasyExecutor<D>) -> anyhow::Result<Self::Output>;
}
