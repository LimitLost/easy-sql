use std::ops::{Deref, DerefMut};

use anyhow::Context;
use easy_macros::always_context;
use sqlx::Database;
use std::fmt::Debug;

use crate::{Driver, DriverConnection, InternalDriver, SetupSql, easy_executor::EasyExecutor};
#[derive(Debug)]
pub struct Transaction<'a, D: Driver> {
    internal: sqlx::Transaction<'a, D::InternalDriver>,
}

#[always_context]
impl<'a, D: Driver> Transaction<'a, D> {
    pub(crate) fn new(internal: sqlx::Transaction<'a, D::InternalDriver>) -> Self {
        Transaction { internal }
    }

    pub async fn commit(self) -> anyhow::Result<()> {
        self.internal.commit().await?;
        Ok(())
    }

    pub async fn rollback(self) -> anyhow::Result<()> {
        self.internal.rollback().await?;
        Ok(())
    }
}

#[always_context]
impl<'c, D: Driver> EasyExecutor<D> for &mut Transaction<'c, D>
where
    for<'b> &'b mut DriverConnection<D>: sqlx::Executor<'b, Database = D::InternalDriver>,
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
impl<'c, D: Driver> Deref for Transaction<'c, D> {
    type Target = <InternalDriver<D> as Database>::Connection;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<'c, D: Driver> DerefMut for Transaction<'c, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}
