use std::sync::Arc;

use anyhow::Context;
use easy_macros::macros::always_context;

use tokio::sync::Mutex;

use crate::{Connection, DatabaseInternal, DatabaseSetup, EasySqlTables, Transaction};

use super::Db;

pub use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use super::{DatabaseInternalDefault, Postgres};

#[derive(Debug)]
pub struct Database<DI: DatabaseInternal<Driver = Postgres> + Send + Sync = DatabaseInternalDefault>
{
    connection_pool: sqlx::Pool<Db>,
    internal: Arc<Mutex<DI>>,
}

#[always_context]
impl<DI: DatabaseInternal<Driver = Postgres> + Send + Sync> Database<DI> {
    pub async fn setup<T: DatabaseSetup<Postgres>>(
        url: &str,
        internal: Arc<Mutex<DI>>,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect(url).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?, internal.clone());

        EasySqlTables::setup(&mut conn).await?;
        T::setup(&mut conn).await?;

        Ok(Database {
            connection_pool,
            internal,
        })
    }

    pub async fn setup_with_options<T: DatabaseSetup<Postgres>>(
        options: PgConnectOptions,
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

    pub async fn conn(&self) -> anyhow::Result<Connection<Postgres, DI>> {
        let conn = self.connection_pool.acquire().await?;
        Ok(Connection::new(conn, self.internal.clone()))
    }

    pub async fn transaction(&self) -> anyhow::Result<Transaction<'_, Postgres, DI>> {
        let conn = self.connection_pool.begin().await?;
        Ok(Transaction::new(conn, self.internal.clone()))
    }
}

#[always_context]
impl Database<DatabaseInternalDefault> {
    #[cfg(test)]
    pub async fn setup_for_testing<T: DatabaseSetup<Postgres>>() -> anyhow::Result<Self> {
        lazy_static::lazy_static! {
            static ref CURRENT_NAME_N:Mutex<usize>=Default::default();
        }

        // Load environment variables from .env file
        let _ = dotenvy::dotenv();

        let host = std::env::var("POSTGRES_HOST")
            .context("POSTGRES_HOST .env variable must be set for tests")?;
        let port: u16 = std::env::var("POSTGRES_PORT")
            .context("POSTGRES_PORT .env variable must be set for tests")?
            .parse()
            .context("Invalid POSTGRES_PORT")?;
        let username = std::env::var("POSTGRES_USER")
            .context("POSTGRES_USER .env variable must be set for tests")?;
        let password = std::env::var("POSTGRES_PASSWORD")
            .context("POSTGRES_PASSWORD .env variable must be set for tests")?;
        let db_prefix = std::env::var("POSTGRES_TEST_DB_PREFIX")
            .context("POSTGRES_TEST_DB_PREFIX .env variable must be set for tests")?;

        let mut current_n = CURRENT_NAME_N.lock().await;
        let test_database = format!("{}_{}", db_prefix, *current_n);
        *current_n += 1;

        // Recreate test database
        let maintenance_pool = sqlx::Pool::<Db>::connect_with(
            PgConnectOptions::new()
                .host(&host)
                .port(port)
                .username(&username)
                .password(&password),
        )
        .await?;

        sqlx::query(&format!("DROP DATABASE IF EXISTS {}", test_database))
            .execute(&maintenance_pool)
            .await?;

        sqlx::query(&format!("CREATE DATABASE {}", test_database))
            .execute(&maintenance_pool)
            .await?;

        maintenance_pool.close().await;

        // Connect to the test database
        let connection_pool = sqlx::Pool::<Db>::connect_with(
            PgConnectOptions::new()
                .host(&host)
                .port(port)
                .username(&username)
                .password(&password)
                .database(&test_database),
        )
        .await?;

        let internal: Arc<Mutex<DatabaseInternalDefault>> =
            Arc::new(Mutex::new(DatabaseInternalDefault {
                test_db_file_path: None, // Postgres doesn't use file paths
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
