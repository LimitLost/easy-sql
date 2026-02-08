use easy_macros::always_context;
use sqlx::{Pool, SqliteConnection};

use super::Db;
use crate::{
    EasyExecutor,
    traits::{DriverConnection, SetupSql},
};

type CDriver = super::Sqlite;
/// For some reason Db::Connection overlaps with crate::Connection
type Connection = SqliteConnection;

#[always_context(skip(!))]
impl EasyExecutor<CDriver> for &Pool<Db> {
    type InternalExecutor<'b>
        = &'b Pool<Db>
    where
        Self: 'b;
    type IntoInternalExecutor<'b>
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
    type IntoInternalExecutor<'b>
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

    fn into_executor<'a>(self) -> Self::IntoInternalExecutor<'a>
    where
        Self: 'a,
    {
        self
    }
}
