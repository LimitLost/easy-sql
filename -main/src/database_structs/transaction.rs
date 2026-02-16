use std::ops::{Deref, DerefMut};

use anyhow::Context;
use easy_macros::always_context;
use sqlx::Database;
use std::fmt::Debug;

use crate::{
    Driver, EasyExecutor,
    traits::{DriverConnection, InternalDriver, SetupSql},
};
/// Wrapper around [`sqlx::Transaction`](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html)
///
/// Will contain sql query watch data in the future (gated by a feature)
#[derive(Debug)]
pub struct Transaction<'a, D: Driver> {
    internal: sqlx::Transaction<'a, D::InternalDriver>,
}

#[always_context]
impl<'a, D: Driver> Transaction<'a, D> {
    pub fn new(internal: sqlx::Transaction<'a, D::InternalDriver>) -> Self {
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
impl<'c, D: Driver> EasyExecutor<D> for Transaction<'c, D>
where
    for<'b> &'b mut DriverConnection<D>: sqlx::Executor<'b, Database = D::InternalDriver>,
{
    type InternalExecutor<'b>
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
/// Wrapper around [`sqlx::Transaction`](https://docs.rs/sqlx/latest/sqlx/struct.Transaction.html)
///
/// Represents a transaction started from a connection pool, so it can be sent across threads and awaited on without holding up the connection.
///
/// Will contain sql query watch data in the future (gated by a feature)
#[derive(Debug)]
pub struct PoolTransaction<D: Driver> {
    internal: sqlx::Transaction<'static, D::InternalDriver>,
}

#[always_context]
impl<D: Driver> PoolTransaction<D> {
    pub fn new(internal: sqlx::Transaction<'static, D::InternalDriver>) -> Self {
        PoolTransaction { internal }
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
impl<D: Driver> EasyExecutor<D> for PoolTransaction<D>
where
    for<'b> &'b mut DriverConnection<D>: sqlx::Executor<'b, Database = D::InternalDriver>,
{
    type InternalExecutor<'b>
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
}
impl<D: Driver> Deref for PoolTransaction<D> {
    type Target = <InternalDriver<D> as Database>::Connection;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<D: Driver> DerefMut for PoolTransaction<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}
