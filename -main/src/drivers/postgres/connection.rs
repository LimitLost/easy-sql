use easy_macros::always_context;
use sqlx::{PgConnection, Pool};

use super::Db;
use crate::{
    EasyExecutor, EasyExecutorInto,
    traits::{DriverConnection, SetupSql},
};

type CDriver = super::Postgres;
/// For some reason Db::Connection overlaps with crate::Connection
type Connection = PgConnection;

#[always_context(skip(!))]
impl EasyExecutor<CDriver> for &Pool<Db> {
    type InternalExecutor<'b>
        = &'b Pool<Db>
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<CDriver> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<CDriver>: Send + Sync,
    {
        sql.query(self).await
    }

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a> {
        self
    }
}

impl EasyExecutorInto<CDriver> for &Pool<Db> {
    type IntoInternalExecutor<'b>
        = &'b Pool<Db>
    where
        Self: 'b;

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        self
    }
}

#[always_context(skip(!))]
impl EasyExecutor<CDriver> for &mut Connection {
    type InternalExecutor<'b>
        = &'b mut Connection
    where
        Self: 'b;

    async fn query_setup<O: SetupSql<CDriver> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<CDriver>: Send + Sync,
    {
        sql.query(self).await
    }

    fn executor<'a>(&'a mut self) -> Self::InternalExecutor<'a> {
        self
    }
}

impl EasyExecutorInto<CDriver> for &mut Connection {
    type IntoInternalExecutor<'b>
        = &'b mut Connection
    where
        Self: 'b;

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        self
    }
}
