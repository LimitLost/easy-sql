use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use easy_macros::always_context;

use sqlx::{Database, Executor};

use crate::{Driver, DriverConnection, InternalDriver, SetupSql, easy_executor::EasyExecutor};

#[derive(Debug)]
pub struct Connection<D: Driver> {
    internal: sqlx::pool::PoolConnection<InternalDriver<D>>,
}

#[always_context]
impl<D: Driver> Connection<D> {
    pub(crate) fn new(conn: sqlx::pool::PoolConnection<InternalDriver<D>>) -> Self {
        Connection { internal: conn }
    }
}

#[always_context]
impl<D: Driver> EasyExecutor<D> for &mut Connection<D>
where
    for<'b> &'b mut DriverConnection<D>: Executor<'b, Database = D::InternalDriver>,
{
    type InternalExecutor<'b>
        = &'b mut DriverConnection<D>
    where
        Self: 'b;

    type IntoInternalExecutor<'b>
        = &'b mut DriverConnection<D>
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
        &mut *self.internal
    }

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        &mut *self.internal
    }
}

impl<D: Driver> Deref for Connection<D> {
    type Target = <InternalDriver<D> as Database>::Connection;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<D: Driver> DerefMut for Connection<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}
