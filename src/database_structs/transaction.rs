use std::sync::Arc;

use anyhow::Context;
use easy_macros::macros::always_context;
use futures::StreamExt;
use std::fmt::Debug;
use tokio::sync::Mutex;

use crate::{
    DatabaseInternal, Driver, DriverArguments, DriverConnection, DriverRow, InternalDriver,
    SetupSql, Sql, SqlOutput, ToConvert,
    easy_executor::{Break, EasyExecutor},
};
#[derive(Debug)]
pub struct Transaction<'a, D: Driver, DI: DatabaseInternal<Driver = D>> {
    internal: sqlx::Transaction<'a, D::InternalDriver>,
    db_link: Arc<Mutex<DI>>,
}

#[always_context]
impl<'a, D: Driver, DI: DatabaseInternal<Driver = D>> Transaction<'a, D, DI> {
    pub(crate) fn new(
        internal: sqlx::Transaction<'a, D::InternalDriver>,
        db_link: Arc<Mutex<DI>>,
    ) -> Self {
        Transaction { internal, db_link }
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
impl<'c, DI: DatabaseInternal<Driver = D> + Send + Sync, D: Driver> EasyExecutor<D>
    for Transaction<'c, D, DI>
{
    /* async fn query(&mut self, sql: &Sql) -> anyhow::Result<()> {
           sql.query()?.sqlx().execute(&mut *self.internal).await?;

           //Inform about query DatabaseInternal
           #[no_context_inputs]
           self.db_link
               .lock()
               .await
               .sql_request(&mut *self.internal, sql)
               .await?;

           Ok(())
       }
    */
    async fn query<Y: ToConvert<D> + Send + Sync, T, O: SqlOutput<T, D, Y>>(
        &mut self,
        sql: &Sql<'_, D>,
    ) -> anyhow::Result<O>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = InternalDriver<D>> + Send + Sync,
    {
        // SAFETY: We need to unify lifetimes for sql_to_query call.
        // This is safe because we're not extending lifetimes, just unifying them
        // for the duration of this function call.
        let sql_unified: &Sql<'_, D> = unsafe { std::mem::transmute(sql) };
        let query = O::sql_to_query(sql_unified)?;
        let query_sqlx = query.sqlx();

        /* let conn_ref: &'a mut DriverConnection<CurrentDriver> =
        unsafe { &mut *(&mut *self.internal as *mut DriverConnection<CurrentDriver>) }; */

        #[no_context_inputs]
        let row = Y::get(&mut *self.internal, query_sqlx)
            .await
            .with_context(|| {
                let query = O::sql_to_query(sql_unified);
                format!("query: {query:?}")
            })?;

        //Inform about query DatabaseInternal
        #[no_context_inputs]
        self.db_link
            .lock()
            .await
            .sql_request(sql)
            .await
            .with_context(|| {
                let query = O::sql_to_query(sql_unified);
                format!("query: {query:?}")
            })?;
        #[no_context_inputs]
        Ok(O::convert(row).with_context(|| {
            let query = O::sql_to_query(sql_unified);
            format!("Converting Row to Value | query: {query:?}")
        })?)
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

    /* async fn fetch_all<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<Vec<O>> {
        let rows = sql
            .query_output(O::requested_columns())?
            .sqlx()
            .fetch_all(&mut *self.internal)
            .await?;

        //Inform about query DatabaseInternal
        #[no_context_inputs]
        self.db_link
            .lock()
            .await
            .sql_request(&mut *self.internal, sql)
            .await?;

        #[no_context_inputs]
        Ok(<Vec<O> as SqlOutput<T, Vec<Row>>>::convert(rows)?)
    } */

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
        mut perform: impl FnMut(O) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = InternalDriver<D>> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, InternalDriver<D>>,
    {
        // SAFETY: We need to unify lifetimes for sql_to_query call.
        // This is safe because we're not extending lifetimes, just unifying them
        // for the duration of this function call.
        let sql_unified: &Sql<'_, D> = unsafe { std::mem::transmute(sql) };
        let query_output = O::sql_to_query(sql_unified)?;

        // SAFETY: Extending lifetime to 'a for sql_request
        // This is safe because the connection outlives this function call
        // let conn_ref: &'a mut DriverConnection<D> =
        //     unsafe { &mut *(&mut *self.internal as *mut DriverConnection<D>) };

        //Inform about query DatabaseInternal
        #[no_context_inputs]
        self.db_link
            .lock()
            .await
            .sql_request(sql)
            .await
            .with_context(|| {
                let query_output = O::sql_to_query(sql_unified);
                format!("Row fetching | query: {query_output:?}")
            })?;

        // Create fresh borrow for fetch
        // let conn_ref2: &'a mut DriverConnection<D> =
        //     unsafe { &mut *(&mut *self.internal as *mut DriverConnection<D>) };

        let mut rows = query_output.sqlx().fetch(&mut *self.internal);

        while let Some(result) = rows.next().await {
            let row = result.with_context(|| {
                let query_output = O::sql_to_query(sql_unified);
                format!("Row fetching | query: {query_output:?}")
            })?;
            #[no_context_inputs]
            let output = O::convert(row).with_context(|| {
                let query_output = O::sql_to_query(sql_unified);
                format!("Converting Row to Value | query: {query_output:?}")
            })?;

            #[no_context_inputs]
            if let Some(Break) = perform(output)? {
                break;
            }
        }

        Ok(())
    }
}
