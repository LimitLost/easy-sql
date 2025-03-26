use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;
use easy_macros::{helpers::context, macros::always_context};
use tokio::sync::Mutex;

use super::DatabaseInternal;
use crate::{ConnectionInternal, Row, Sql, SqlOutput, easy_executor::EasyExecutor};

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
    async fn query(&mut self, sql: &Sql) -> anyhow::Result<()> {
        sql.query()?.sqlx().execute(&mut *self.internal).await?;

        Ok(())
    }

    async fn fetch_one<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<O> {
        todo!()
    }

    async fn fetch_many<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<Vec<O>> {
        todo!()
    }
}
