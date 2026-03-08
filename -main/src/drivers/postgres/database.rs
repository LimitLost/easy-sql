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
        use crate::tests::init_test_logger;

        init_test_logger();

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

        let test_database = generate_postgres_test_database_name(&db_prefix)?;
        recreate_postgres_test_database(&host, port, &username, &password, &test_database).await?;

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

#[cfg(test)]
pub(crate) use test::*;
#[cfg(test)]
mod test {
    use std::sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    };

    use super::Db;
    use anyhow::{Context, ensure};
    use easy_macros::always_context;
    use sqlx::postgres::PgConnectOptions;

    pub(crate) const POSTGRES_DB_NAME_MAX_BYTES: usize = 63;

    static POSTGRES_TEST_DB_NAME_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    static POSTGRES_TEST_DB_NAME_SALT: OnceLock<u64> = OnceLock::new();

    fn postgres_test_db_name_salt() -> u64 {
        *POSTGRES_TEST_DB_NAME_SALT.get_or_init(|| {
            let startup_nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map_or(0, |duration| duration.as_nanos() as u64);
            let pid = u64::from(std::process::id());
            startup_nanos ^ pid.rotate_left(13) ^ 0x9e37_79b9_7f4a_7c15
        })
    }

    fn truncate_utf8_prefix(input: &str, max_bytes: usize) -> &str {
        if input.len() <= max_bytes {
            return input;
        }

        let mut end = 0;
        for (index, ch) in input.char_indices() {
            let next = index + ch.len_utf8();
            if next > max_bytes {
                break;
            }
            end = next;
        }
        &input[..end]
    }

    /// Generates a PostgreSQL test database name that always fits the 63-byte identifier limit.
    ///
    /// The function appends a unique suffix (`pool_<salt>_<pid>_<sequence>`) and truncates
    /// the prefix on UTF-8 boundaries so multibyte characters are never cut in the middle.
    #[always_context(skip(!))]
    pub(crate) fn generate_postgres_test_database_name(db_prefix: &str) -> anyhow::Result<String> {
        let salt = postgres_test_db_name_salt();
        let pid = u64::from(std::process::id());
        let sequence = POSTGRES_TEST_DB_NAME_SEQUENCE.fetch_add(1, Ordering::Relaxed);

        let suffix = format!("pool_{salt:016x}_{pid:08x}_{sequence:016x}");

        let max_prefix_len = POSTGRES_DB_NAME_MAX_BYTES.saturating_sub(suffix.len() + 1);
        let truncated_prefix = truncate_utf8_prefix(db_prefix, max_prefix_len);

        let test_database = if truncated_prefix.is_empty() {
            suffix
        } else {
            format!("{truncated_prefix}_{suffix}")
        };

        ensure!(
            test_database.len() <= POSTGRES_DB_NAME_MAX_BYTES,
            "Generated PostgreSQL test database name exceeds {} bytes (prefix_bytes={}, result_bytes={}): {}",
            POSTGRES_DB_NAME_MAX_BYTES,
            db_prefix.len(),
            test_database.len(),
            test_database
        );

        Ok(test_database)
    }

    /// Recreates a PostgreSQL test database via a maintenance connection.
    ///
    /// Connects without selecting a database, then runs:
    /// 1. `DROP DATABASE IF EXISTS "<escaped_name>"`
    /// 2. `CREATE DATABASE "<escaped_name>"`
    ///
    /// Identifier escaping is preserved by doubling embedded quotes in `test_database`.
    #[always_context(skip(!))]
    pub(crate) async fn recreate_postgres_test_database(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        test_database: &str,
    ) -> anyhow::Result<()> {
        let maintenance_pool = sqlx::Pool::<Db>::connect_with(
            PgConnectOptions::new()
                .host(host)
                .port(port)
                .username(username)
                .password(password),
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
        Ok(())
    }
}
