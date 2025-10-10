use std::{fmt::Debug, ops::DerefMut};

use easy_macros::macros::always_context;

use crate::{
    Driver, DriverArguments, DriverConnection, DriverRow, SqlOutput, SqlTable, ToConvert,
    sql_query::Sql,
};

pub struct Break;

#[always_context]
pub trait EasyExecutor<D: Driver>: Debug {
    // async fn query(&mut self, sql: &Sql) -> anyhow::Result<()>;
    async fn query<Y: ToConvert<D> + Send + Sync, T: SqlTable<D>, O: SqlOutput<T, D, Y>>(
        &mut self,
        sql: &Sql<'_, D>,
    ) -> anyhow::Result<O>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync;

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync;

    // async fn fetch_all<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<Vec<O>>;

    ///# How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn fetch_lazy<T, O: SqlOutput<T, D, DriverRow<D>>>(
        &mut self,
        sql: &Sql<'_, D>,
        perform: impl FnMut(O) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>;
}

#[always_context]
pub trait SetupSql<D: Driver> {
    type Output;

    async fn query(
        self,
        exec: &mut (
                 impl DerefMut<Target = <D::InternalDriver as sqlx::database::Database>::Connection>
                 + Send
                 + Sync
             ),
    ) -> anyhow::Result<Self::Output>;
}
