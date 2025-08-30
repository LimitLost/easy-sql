#[cfg(test)]
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use easy_macros::macros::always_context;
use sqlx::Executor;

use std::path::Path;
use tokio::sync::Mutex;

use super::{Connection, Transaction};
use crate::{DatabaseSetup, Db, EasySqlTables, sql_query::Sql};

/// TODO Will be used in the future to send data to the remote database
#[derive(Debug, Default)]
pub(crate) struct DatabaseInternal {
    #[cfg(test)]
    test_db_file_path: Option<PathBuf>,
}
#[cfg(test)]
impl Drop for DatabaseInternal {
    fn drop(&mut self) {
        if let Some(path) = &self.test_db_file_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

pub use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

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
#[derive(Debug)]
pub struct Database {
    connection_pool: sqlx::Pool<Db>,
    internal: Arc<Mutex<DatabaseInternal>>,
}

#[always_context]
impl Database {
    pub async fn setup<T: DatabaseSetup>(db_file_path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&db_file_path)
                .create_if_missing(true),
        )
        .await?;

        let internal: Arc<Mutex<DatabaseInternal>> = Default::default();

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    pub async fn setup_with_options<T: DatabaseSetup>(
        options: SqliteConnectOptions,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(options.clone()).await?;

        let internal: Arc<Mutex<DatabaseInternal>> = Default::default();

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    pub async fn setup_in_memory<T: DatabaseSetup>() -> anyhow::Result<Self> {
        let connection_pool =
            sqlx::Pool::<Db>::connect_with(SqliteConnectOptions::default().in_memory(true)).await?;

        let internal: Arc<Mutex<DatabaseInternal>> = Default::default();
        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    #[cfg(test)]
    pub async fn setup_for_testing<T: DatabaseSetup>() -> anyhow::Result<Self> {
        lazy_static::lazy_static! {
            static ref CURRENT_NAME_N:Mutex<usize>=Default::default();
        }
        let current_path = std::env::current_dir()?;
        let mut current_n = CURRENT_NAME_N.lock().await;
        let test_db_path = current_path.join(format!("test_db_{}", *current_n));
        *current_n += 1;

        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&test_db_path)
                .create_if_missing(true),
        )
        .await?;

        let internal: Arc<Mutex<DatabaseInternal>> = Arc::new(Mutex::new(DatabaseInternal {
            test_db_file_path: Some(test_db_path.clone()),
        }));

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
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
