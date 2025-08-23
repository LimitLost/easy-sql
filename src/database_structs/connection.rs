use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use easy_macros::macros::always_context;
use futures::{StreamExt, TryStreamExt};

use tokio::sync::Mutex;

use super::DatabaseInternal;
use crate::{
    ConnectionInternal, Row, SetupSql, Sql, SqlOutput, ToConvert,
    easy_executor::{Break, EasyExecutor},
};
#[derive(Debug)]
pub struct Connection {
    internal: ConnectionInternal,
    db_link: Arc<Mutex<DatabaseInternal>>,
}

#[always_context]
impl Connection {
    pub(crate) fn new(conn: ConnectionInternal, db_link: Arc<Mutex<DatabaseInternal>>) -> Self {
        Connection {
            internal: conn,
            db_link,
        }
    }
}

#[always_context]
#[async_trait]
impl EasyExecutor for Connection {
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
    async fn query<Y: ToConvert + Send + Sync, T, O: SqlOutput<T, Y>>(
        &mut self,
        sql: &Sql,
    ) -> anyhow::Result<O> {
        let query = O::sql_to_query(sql)?;
        let query_sqlx = query.sqlx();
        #[no_context_inputs]
        let row = Y::get(&mut *self.internal, query_sqlx)
            .await
            .with_context(|| format!("query: {query:?}"))?;

        //Inform about query DatabaseInternal
        #[no_context_inputs]
        self.db_link
            .lock()
            .await
            .sql_request(&mut *self.internal, sql)
            .await
            .with_context(|| format!("query: {query:?}"))?;

        #[no_context_inputs]
        Ok(O::convert(row)
            .with_context(|| format!("Converting Row to Value | query: {query:?}"))?)
    }

    async fn query_setup<O: SetupSql + Send + Sync>(
        &mut self,
        sql: O,
    ) -> anyhow::Result<O::Output> {
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
    async fn fetch_lazy<T, O: SqlOutput<T, Row>>(
        &mut self,
        sql: &Sql,
        mut perform: impl FnMut(O) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()> {
        let query_output = O::sql_to_query(sql)?;

        //Inform about query DatabaseInternal
        #[no_context_inputs]
        self.db_link
            .lock()
            .await
            .sql_request(&mut *self.internal, sql)
            .await
            .with_context(|| format!("query: {query_output:?}"))?;

        let rows = query_output.sqlx().fetch(&mut *self.internal);

        let mut mapped = rows.map(|e| {
            e.with_context(|| format!("Row fetching | query: {query_output:?}"))
                .and_then(|row| {
                    O::convert(row).with_context(|| {
                        format!("Converting Row to Value | query: {query_output:?}")
                    })
                })
        });

        while let Some(row) = mapped.try_next().await? {
            #[no_context_inputs]
            if let Some(Break) = perform(row)? {
                break;
            }
        }

        Ok(())
    }
}
