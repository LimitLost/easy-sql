use std::sync::Arc;

use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use tokio::sync::Mutex;

use super::{Connection, Transaction};
use crate::{DatabaseSetup, Db, easy_executor::EasyExecutor, sql_query::Sql};

/// TODO Will be used in the future to send data to the remote database
#[derive(Debug, Default)]
pub(crate) struct DatabaseInternal;

#[always_context]
impl DatabaseInternal {
    pub async fn sql_request(&mut self, conn: impl EasyExecutor, sql: &Sql) {
        //TODO Save it for later in the sqlite database
    }
    pub async fn conn_end(&mut self) {
        //TODO Every 1? minute send updates to the remote database
    }
    ///Should be used when user wants to exit the program
    pub async fn maybe_exit(&mut self) {
        //TODO Try sending data to server if not sent yet
    }
}

pub struct Database {
    connection_pool: sqlx::Pool<Db>,
    internal: Arc<Mutex<DatabaseInternal>>,
}
#[always_context]
impl Database {
    pub async fn setup<T: DatabaseSetup>(url: &str) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect(url).await?;
        Ok(Database {
            connection_pool,
            internal: Default::default(),
        })
    }

    pub async fn maybe_exit(&self) {
        let mut internal = self.internal.lock().await;
        internal.maybe_exit().await;
    }

    pub async fn conn(&self) -> anyhow::Result<Connection> {
        let conn = self.connection_pool.acquire().await?;
        Ok(Connection::new(conn, self.internal.clone()))
    }

    pub async fn transaction(&self) -> anyhow::Result<Transaction> {
        let conn = self.connection_pool.begin().await?;
        Ok(Transaction::new(conn, self.internal.clone()))
    }
}
