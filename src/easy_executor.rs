use easy_macros::always_context;

use crate::{Driver, DriverConnection};

#[always_context]
pub trait EasyExecutor<D: Driver> {
    type InternalExecutor<'b>: sqlx::Executor<'b, Database = D::InternalDriver>
    where
        Self: 'b;
    type IntoInternalExecutor<'b>: sqlx::Executor<'b, Database = D::InternalDriver>
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync;

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a>;

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a;
}

#[always_context(skip(!))]
impl<T, D: Driver> EasyExecutor<D> for &T
where
    for<'b> &'b T: sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
{
    type InternalExecutor<'b>
        = &'b T
    where
        Self: 'b;
    type IntoInternalExecutor<'b>
        = &'b T
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync,
    {
        sql.query(self).await
    }

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a> {
        self
    }

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[always_context(skip(!))]
impl<T, D: Driver> EasyExecutor<D> for &mut T
where
    for<'b> &'b mut T: sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
{
    type InternalExecutor<'b>
        = &'b mut T
    where
        Self: 'b;
    type IntoInternalExecutor<'b>
        = &'b mut T
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync,
    {
        sql.query(self).await
    }

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a> {
        self
    }

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[always_context]
pub trait SetupSql<D: Driver> {
    type Output;

    async fn query(self, exec: &mut impl EasyExecutor<D>) -> anyhow::Result<Self::Output>;
}

#[cfg(test)]
#[allow(dead_code)]
mod impl_test {
    use crate::Driver;

    use super::EasyExecutor;

    #[cfg(feature = "postgres")]
    type CurrentDriver = sqlx::Postgres;
    #[cfg(feature = "postgres")]
    type CurrentCDriver = crate::Postgres;
    #[cfg(feature = "sqlite")]
    type CurrentDriver = sqlx::Sqlite;
    #[cfg(feature = "sqlite")]
    type CurrentCDriver = crate::Sqlite;

    /// Both sqlx pool and connection should have this trait auto implemented
    fn impl_test_base<D: Driver>(_exe: impl EasyExecutor<D>) {}

    fn impl_test(
        pool: sqlx::Pool<CurrentDriver>,
        mut conn: <CurrentDriver as sqlx::Database>::Connection,
    ) {
        impl_test_base::<CurrentCDriver>(&pool);
        impl_test_base::<CurrentCDriver>(&mut conn);
    }
}
