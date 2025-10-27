use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use anyhow::Context;
use easy_macros::macros::always_context;
use futures::StreamExt;

use sqlx::Database;
use tokio::sync::Mutex;

use crate::{
    DatabaseInternal, Driver, DriverArguments, DriverConnection, DriverRow, InternalDriver, Output,
    QueryBuilder, SetupSql, Sql, ToConvert,
    easy_executor::{Break, EasyExecutor},
};

#[derive(Debug)]
pub struct Connection<D: Driver, DI: DatabaseInternal<Driver = D>> {
    internal: sqlx::pool::PoolConnection<InternalDriver<D>>,
    db_link: Arc<Mutex<DI>>,
}

#[always_context]
impl<D: Driver, DI: DatabaseInternal<Driver = D>> Connection<D, DI> {
    pub(crate) fn new(
        conn: sqlx::pool::PoolConnection<InternalDriver<D>>,
        db_link: Arc<Mutex<DI>>,
    ) -> Self {
        Connection {
            internal: conn,
            db_link,
        }
    }
}

#[always_context]
impl<D: Driver, DI: DatabaseInternal<Driver = D> + Send + Sync> EasyExecutor<D>
    for Connection<D, DI>
{
    async fn query<Y: ToConvert<D> + Send + Sync + 'static, T, O: Output<T, D, DataToConvert = Y>>(
        &mut self,
        sql: Sql,
        builder: QueryBuilder<'_, D>,
    ) -> anyhow::Result<O>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = InternalDriver<D>> + Send + Sync,
        for<'b> DriverArguments<'b, D>: Debug,
    {
        // SAFETY: Compiler thinks that that data inside of the builder could be borrowed forever, entangled with connection
        // Because of Y::get, see SAFETY below
        let builder: QueryBuilder<'_, D> = unsafe { std::mem::transmute(builder) };

        // TODO Inform QueryListener

        let mut query = O::sql_to_query(
            #[context(no)]
            sql,
            #[context(no)]
            builder,
        )?;
        // SAFETY: We transmute the query to  satisfy compiler thinking that Y::get borrows query_sqlx forever.
        // This is safe because:
        // 1. Y: 'static ensures the returned type doesn't contain non-'static references (so it doesn't borrow forever)
        let query_sqlx = unsafe { query.sqlx() };

        #[no_context_inputs]
        let row = Y::get(&mut *self.internal, query_sqlx).await?;

        #[no_context_inputs]
        Ok(O::convert(row)?)
    }

    async fn query_setup<O: SetupSql<D> + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output>
    where
        DriverConnection<D>: Send + Sync,
    {
        sql.query(&mut self.internal).await
    }

    ///# How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn fetch_lazy<T, O: Output<T, D, DataToConvert = DriverRow<D>>>(
        &mut self,
        sql: Sql,
        builder: QueryBuilder<'_, D>,
        mut perform: impl FnMut(O) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D> + 'static,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = InternalDriver<D>> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, InternalDriver<D>> + Debug,
    {
        // SAFETY: Compiler thinks that that data inside of the builder could be borrowed forever, entangled with connection
        // Because of Y::get, see SAFETY below
        let builder: QueryBuilder<'_, D> = unsafe { std::mem::transmute(builder) };

        // TODO Inform QueryListener

        // sql context to result is added in the Table invocations
        let mut query_output = O::sql_to_query(
            #[context(no)]
            sql,
            #[context(no)]
            builder,
        )?;

        // SAFETY: We transmute the query to  satisfy compiler thinking that Y::get borrows query_sqlx forever.
        // This is safe because:
        // 1. DriverRow<D>: 'static ensures the generated Row doesn't contain non-'static references (so it doesn't borrow forever our sqlx query)
        let mut rows = unsafe { query_output.sqlx() }.fetch(&mut *self.internal);

        while let Some(result) = rows.next().await {
            let row = result.context("Failed to fetch row")?;
            #[no_context_inputs]
            let output = O::convert(row)?;

            #[no_context_inputs]
            if let Some(Break) = perform(output)? {
                break;
            }
        }

        Ok(())
    }
}

impl<DI: DatabaseInternal<Driver = D> + Send + Sync, D: Driver> Deref for Connection<D, DI> {
    type Target = <InternalDriver<D> as Database>::Connection;

    fn deref(&self) -> &Self::Target {
        &self.internal
    }
}

impl<DI: DatabaseInternal<Driver = D> + Send + Sync, D: Driver> DerefMut for Connection<D, DI> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.internal
    }
}
