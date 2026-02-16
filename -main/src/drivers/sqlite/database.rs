use anyhow::Context;
use easy_macros::always_context;

use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use crate::{Connection, DatabaseSetup, EasySqlTables, Transaction};

use super::Db;

pub use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

use super::Sqlite;

/// SQLite connection pool wrapper with setup helpers.
///
/// Uses [`DatabaseSetup`](crate::DatabaseSetup) implementations to prepare schema on startup.
#[derive(Debug)]
pub struct Database {
    connection_pool: sqlx::Pool<Db>,
    #[cfg(test)]
    pub test_db_file_path: Option<PathBuf>,
}

#[cfg(test)]
impl Drop for Database {
    fn drop(&mut self) {
        if let Some(path) = &self.test_db_file_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[always_context]
impl Database {
    pub async fn setup<T: DatabaseSetup<Sqlite>>(
        db_file_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&db_file_path)
                .create_if_missing(true),
        )
        .await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: Some(db_file_path.as_ref().to_path_buf()),
        })
    }

    pub async fn setup_with_options<T: DatabaseSetup<Sqlite>>(
        options: SqliteConnectOptions,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(options.clone()).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: Some(options.get_filename().to_owned()),
        })
    }

    // Broken - database will be lost after connection is closed
    /* pub async fn setup_in_memory<T: DatabaseSetup<Sqlite>>() -> anyhow::Result<Self> {
        let connection_pool =
            sqlx::Pool::<Db>::connect_with(SqliteConnectOptions::default().in_memory(true)).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: None,
        })
    } */

    pub async fn conn(&self) -> anyhow::Result<Connection<Sqlite>> {
        let conn = self.connection_pool.acquire().await?;
        Ok(Connection::new(conn))
    }

    pub async fn transaction(&self) -> anyhow::Result<PoolTransaction<Sqlite>> {
        let conn = self.connection_pool.begin().await?;
        Ok(PoolTransaction::new(conn))
    }
    #[cfg(test)]
    pub async fn setup_for_testing<T: DatabaseSetup<Sqlite>>() -> anyhow::Result<Self> {
        use tokio::sync::Mutex;

        use crate::tests::init_test_logger;

        init_test_logger();

        lazy_static::lazy_static! {
            static ref CURRENT_NAME_N:Mutex<usize>=Default::default();
        }
        let current_path = std::env::current_dir()?;
        let test_db_path = {
            let mut current_n = CURRENT_NAME_N.lock().await;
            let path = current_path.join(format!("test_db_{}", *current_n));
            *current_n += 1;
            path
        };

        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&test_db_path)
                .create_if_missing(true),
        )
        .await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            test_db_file_path: Some(test_db_path),
        })
    }
}
