use std::sync::Arc;

use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::Executor;
use tokio::sync::Mutex;

use super::{Connection, Transaction};
use crate::{DatabaseSetup, Db, sql_query::Sql};

/// TODO Will be used in the future to send data to the remote database
#[derive(Debug, Default)]
pub(crate) struct DatabaseInternal;

#[always_context]
impl DatabaseInternal {
    pub async fn sql_request<'a>(
        &mut self,
        conn: impl Executor<'a, Database = Db>,
        sql: &Sql<'a>,
    ) -> anyhow::Result<()> {
        //TODO Save it for later in the sqlite database

        Ok(())
    }
    //TODO Use tokio::spawn in sql_request instead
    /* pub async fn conn_end(&mut self) -> anyhow::Result<()> {
        //Every 1? minute send updates to the remote database
        Ok(())
    } */
    ///Should be used when user wants to exit the program
    pub async fn maybe_exit(&mut self) -> anyhow::Result<()> {
        //TODO Try sending data to server if not sent yet
        Ok(())
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

    pub async fn maybe_exit(&self) -> anyhow::Result<()> {
        let mut internal = self.internal.lock().await;
        internal.maybe_exit().await?;

        Ok(())
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
