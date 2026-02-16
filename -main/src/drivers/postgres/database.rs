use anyhow::Context;
use easy_macros::always_context;

use crate::{Connection, DatabaseSetup, EasySqlTables, PoolTransaction};

use super::Db;

pub use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use super::Postgres;

/// PostgreSQL connection pool wrapper with setup helpers.
///
/// Uses [`DatabaseSetup`](crate::DatabaseSetup) implementations to prepare schema on startup.
#[derive(Debug)]
pub struct Database {
    connection_pool: sqlx::Pool<Db>,
}

#[always_context]
impl Database {
    pub async fn setup<T: DatabaseSetup<Postgres>>(url: &str) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect(url).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database { connection_pool })
    }

    pub async fn setup_with_options<T: DatabaseSetup<Postgres>>(
        options: PgConnectOptions,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(options.clone()).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database { connection_pool })
    }

    pub async fn conn(&self) -> anyhow::Result<Connection<Postgres>> {
        let conn = self.connection_pool.acquire().await?;
        Ok(Connection::new(conn))
    }

    pub async fn transaction(&self) -> anyhow::Result<PoolTransaction<Postgres>> {
        let conn = self.connection_pool.begin().await?;
        Ok(PoolTransaction::new(conn))
    }

    #[cfg(test)]
    pub async fn setup_for_testing<T: DatabaseSetup<Postgres>>() -> anyhow::Result<Self> {
        use tokio::sync::Mutex;

        use crate::tests::init_test_logger;

        init_test_logger();

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

        let test_database = {
            let mut current_n = CURRENT_NAME_N.lock().await;
            let name = format!("{}_{}", db_prefix, *current_n);
            *current_n += 1;
            name
        };

        // Recreate test database
        let maintenance_pool = sqlx::Pool::<Db>::connect_with(
            PgConnectOptions::new()
                .host(&host)
                .port(port)
                .username(&username)
                .password(&password),
        )
        .await?;

        let safe_test_database = test_database.replace('"', "\"\"");

        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS \"{}\"",
            safe_test_database
        ))
        .execute(&maintenance_pool)
        .await?;

        sqlx::query(&format!("CREATE DATABASE \"{}\"", safe_test_database))
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

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database { connection_pool })
    }
}
