use std::sync::Arc;

use anyhow::Context;
use easy_macros::macros::always_context;

use std::path::Path;
use tokio::sync::Mutex;

use crate::{Connection, DatabaseInternal, DatabaseSetup, EasySqlTables, Transaction};

use super::Db;

pub use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

use super::{DatabaseInternalDefault, Sqlite};

#[derive(Debug)]
pub struct Database<DI: DatabaseInternal<Driver = Sqlite> + Send + Sync = DatabaseInternalDefault> {
    connection_pool: sqlx::Pool<Db>,
    internal: Arc<Mutex<DI>>,
}

#[always_context]
impl<DI: DatabaseInternal<Driver = Sqlite> + Send + Sync> Database<DI> {
    pub async fn setup<T: DatabaseSetup<Sqlite>>(
        db_file_path: impl AsRef<Path>,
        internal: Arc<Mutex<DI>>,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&db_file_path)
                .create_if_missing(true),
        )
        .await?;

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    pub async fn setup_with_options<T: DatabaseSetup<Sqlite>>(
        options: SqliteConnectOptions,
        internal: Arc<Mutex<DI>>,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(options.clone()).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    pub async fn setup_in_memory<T: DatabaseSetup<Sqlite>>(
        internal: Arc<Mutex<DI>>,
    ) -> anyhow::Result<Self> {
        let connection_pool =
            sqlx::Pool::<Db>::connect_with(SqliteConnectOptions::default().in_memory(true)).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    pub async fn conn(&self) -> anyhow::Result<Connection<Sqlite, DI>> {
        let conn = self.connection_pool.acquire().await?;
        Ok(Connection::new(conn, self.internal.clone()))
    }

    pub async fn transaction(&self) -> anyhow::Result<Transaction<'_, Sqlite, DI>> {
        let conn = self.connection_pool.begin().await?;
        Ok(Transaction::new(conn, self.internal.clone()))
    }
}

#[always_context]
impl Database<DatabaseInternalDefault> {
    #[cfg(test)]
    pub async fn setup_for_testing<T: DatabaseSetup<Sqlite>>() -> anyhow::Result<Self> {
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

        let internal: Arc<Mutex<DatabaseInternalDefault>> =
            Arc::new(Mutex::new(DatabaseInternalDefault {
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
}
