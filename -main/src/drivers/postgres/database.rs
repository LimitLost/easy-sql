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
        let _ = dotenvy::dotenv();

        // Default to schema-per-test isolation (one shared database, one schema per test): faster, and avoids
        // the per-database create/drop that leaks databases and exhausts connections. Set `EASY_SQL_PG_DEBUG=true`
        // to use the database-per-test path instead — one test maps to one dedicated, inspectable database, which
        // is easier to debug after a failure.
        let per_database_debug = std::env::var("EASY_SQL_PG_DEBUG")
            .map(|v| v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        if per_database_debug {
            Self::setup_per_database_for_test::<T>().await
        } else {
            Self::setup_schema_per_test::<T>().await
        }
    }

    /// Database-per-test isolation: a fresh, dedicated database per test.
    ///
    /// Opt-in debugging aid via `EASY_SQL_PG_DEBUG=true` (one test = one inspectable database), and the
    /// benchmark's per-database baseline, independent of whatever `setup_for_testing` currently defaults to.
    #[cfg(test)]
    #[always_context(skip(!))]
    pub(crate) async fn setup_per_database_for_test<T: DatabaseSetup<Postgres>>() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        let (host, port, username, password) = pg_test_conn_params()?;
        let db_prefix = std::env::var("POSTGRES_TEST_DB_PREFIX")
            .context("POSTGRES_TEST_DB_PREFIX .env variable must be set for tests")?;

        let test_database = generate_postgres_test_database_name(&db_prefix)?;
        recreate_postgres_test_database(&host, port, &username, &password, &test_database).await?;

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

    /// Schema-per-test isolation: one shared database, each test scoped to its own Postgres `SCHEMA`.
    ///
    ///
    /// Gives real isolation (own tables + sequences, so `id == 1` holds) without the per-test
    /// `CREATE`/`DROP DATABASE` cost that makes the per-database path slow and prone to connection-exhaustion
    /// hangs. Every pooled connection is pinned to the schema via `after_connect` `SET search_path`, so existing
    /// test code (`db.transaction()`, `db.conn()`, concurrent `tokio::spawn`) works unchanged.
    #[cfg(test)]
    #[always_context(skip(!))]
    pub async fn setup_schema_per_test<T: DatabaseSetup<Postgres>>() -> anyhow::Result<Self> {
        use crate::tests::init_test_logger;

        init_test_logger();
        let _ = dotenvy::dotenv();

        // - Connection params + the single shared database name.
        let (host, port, username, password) = pg_test_conn_params()?;
        let db_prefix = std::env::var("POSTGRES_TEST_DB_PREFIX")
            .context("POSTGRES_TEST_DB_PREFIX .env variable must be set for tests")?;
        let database = shared_test_db_name();

        // - Create the shared database exactly once per process (race-safe).
        ensure_shared_test_db(&host, port, &username, &password, &database).await?;

        // - Reap leftover schemas/databases from previous runs, exactly once per process. `get_or_try_init`
        // serializes first-callers, so this runs before any test schema exists — it never drops a live one.
        // Assumes one test process per shared database at a time (the test scripts run sequentially).
        static REAPED_ONCE: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
        let reap_result = REAPED_ONCE
            .get_or_try_init(|| async { Self::reap_test_resources().await })
            .await;
        reap_result.context("reaping leftover per-test schemas/databases from a previous run")?;

        // - Unique, 63-byte-safe schema name (reuses the DB-name generator — same identifier rules).
        let schema = generate_postgres_test_database_name(&db_prefix)?;

        // - Per-test pool pinned to the schema. `after_connect` runs on EVERY connection the pool opens
        // (including those a concurrent `tokio::spawn` test acquires), so all of a test's work resolves to its
        // own schema. Bounded connections keep the shared server well under its connection limit.
        let schema_for_hook = schema.clone();
        let connection_pool = PgPoolOptions::new()
            .max_connections(5)
            .min_connections(0)
            .after_connect(move |conn, _meta| {
                let schema = schema_for_hook.clone();
                Box::pin(async move {
                    let safe = schema.replace('"', "\"\"");
                    sqlx::query(&format!("SET search_path TO \"{safe}\""))
                        .execute(&mut *conn)
                        .await?;
                    Ok(())
                })
            })
            .connect_with(
                PgConnectOptions::new()
                    .host(&host)
                    .port(port)
                    .username(&username)
                    .password(&password)
                    .database(&database),
            )
            .await
            // Explicit context also stops `always_context` from inspecting the `after_connect` closure (a closure
            // has no `Debug`), which it otherwise tries to format for the auto-generated context message.
            .context("creating schema-per-test connection pool")?;

        // - Create the schema, then the metadata + application tables inside it (unqualified names
        // resolve via search_path).
        let safe_schema = schema.replace('"', "\"\"");
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS \"{safe_schema}\""))
            .execute(&connection_pool)
            .await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);
        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;
        drop(conn);

        Ok(Database { connection_pool })
    }

    /// Drops leftover per-test resources from this process's prefix: leaked per-test databases and per-test
    /// schemas in the shared database.
    ///
    /// The per-database path leaks databases (no `Drop`), and a crashed
    /// schema-per-test run can leave schemas behind; `WITH (FORCE)` avoids the "database in use" hang.
    #[cfg(test)]
    #[always_context(skip(!))]
    pub(crate) async fn reap_test_resources() -> anyhow::Result<()> {
        let _ = dotenvy::dotenv();
        let (host, port, username, password) = pg_test_conn_params()?;
        let prefix = std::env::var("POSTGRES_TEST_DB_PREFIX")
            .context("POSTGRES_TEST_DB_PREFIX .env variable must be set for tests")?;
        let shared_db = shared_test_db_name();

        let maintenance_pool = sqlx::Pool::<Db>::connect_with(
            PgConnectOptions::new()
                .host(&host)
                .port(port)
                .username(&username)
                .password(&password),
        )
        .await?;

        // Never drop the shared database itself, only the leaked per-test databases.
        let databases: Vec<String> = sqlx::query_scalar(
            "SELECT datname::text FROM pg_database WHERE datname LIKE $1 AND datname <> $2",
        )
        .bind(format!("{prefix}%"))
        .bind(&shared_db)
        .fetch_all(&maintenance_pool)
        .await?;
        for name in &databases {
            let safe = name.replace('"', "\"\"");
            let _ = sqlx::query(&format!("DROP DATABASE IF EXISTS \"{safe}\" WITH (FORCE)"))
                .execute(&maintenance_pool)
                .await;
        }
        maintenance_pool.close().await;

        // Reap leftover per-test schemas inside the shared database (if it exists).
        if let Ok(shared_pool) = sqlx::Pool::<Db>::connect_with(
            PgConnectOptions::new()
                .host(&host)
                .port(port)
                .username(&username)
                .password(&password)
                .database(&shared_db),
        )
        .await
        {
            let schemas: Vec<String> =
                sqlx::query_scalar("SELECT nspname::text FROM pg_namespace WHERE nspname LIKE $1")
                    .bind(format!("{prefix}%"))
                    .fetch_all(&shared_pool)
                    .await
                    .unwrap_or_default();
            for name in &schemas {
                let safe = name.replace('"', "\"\"");
                let _ = sqlx::query(&format!("DROP SCHEMA IF EXISTS \"{safe}\" CASCADE"))
                    .execute(&shared_pool)
                    .await;
            }
            shared_pool.close().await;
        }

        if !databases.is_empty() {
            eprintln!(
                "easy-sql test reaper: dropped {} leftover database(s) matching {}%",
                databases.len(),
                prefix
            );
        }
        Ok(())
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

#[always_context(skip(!))]
    /// Generates a PostgreSQL test database name that always fits the 63-byte identifier limit.
    ///
    /// The function appends a unique suffix (`pool_<salt>_<pid>_<sequence>`) and truncates
    /// the prefix on UTF-8 boundaries so multibyte characters are never cut in the middle.
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

#[always_context(skip(!))]
    /// Recreates a PostgreSQL test database via a maintenance connection.
    ///
    /// Connects without selecting a database, then runs:
    /// 1. `DROP DATABASE IF EXISTS "<escaped_name>"`
    /// 2. `CREATE DATABASE "<escaped_name>"`
    ///
    /// Identifier escaping is preserved by doubling embedded quotes in `test_database`.
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

    /// Guards the one-time creation of the shared schema-per-test database.
    ///
    /// `get_or_try_init` runs the creation on exactly one task even under concurrent first-touch.
    static SHARED_TEST_DB_READY: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

    /// The single database that schema-per-test isolation runs inside (override via `POSTGRES_TEST_DB`).
    pub(crate) fn shared_test_db_name() -> String {
        std::env::var("POSTGRES_TEST_DB").unwrap_or_else(|_| "easy_sql_shared_test".to_string())
    }

    #[always_context(skip(!))]
    /// Reads the four PostgreSQL connection parameters shared by every test-setup path.
    ///
    /// One place for the host/port/user/password lookups instead of repeating them per entry point.
    pub(crate) fn pg_test_conn_params() -> anyhow::Result<(String, u16, String, String)> {
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
        Ok((host, port, username, password))
    }

    #[always_context(skip(!))]
    /// Creates the shared schema-per-test database once per process (idempotent + race-safe).
    ///
    /// The schema-per-test path needs a single database to hold every test's schema; creation must
    /// happen exactly once, and tolerate the database already existing (prior run or a concurrent process).
    pub(crate) async fn ensure_shared_test_db(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
        database: &str,
    ) -> anyhow::Result<()> {
        // Bind the init result first so the init closure is not an operand of a `?`-expression — otherwise
        // `always_context` tries to `Debug`-format the closure for its auto-generated context message.
        let init_result = SHARED_TEST_DB_READY
            .get_or_try_init(|| async {
                let maintenance_pool = sqlx::Pool::<Db>::connect_with(
                    PgConnectOptions::new()
                        .host(host)
                        .port(port)
                        .username(username)
                        .password(password),
                )
                .await?;

                let exists: Option<i32> =
                    sqlx::query_scalar("SELECT 1 FROM pg_database WHERE datname = $1")
                        .bind(database)
                        .fetch_optional(&maintenance_pool)
                        .await?;
                if exists.is_none() {
                    let safe = database.replace('"', "\"\"");
                    // Ignore a lost create race against another process — either way the database now exists.
                    let _ = sqlx::query(&format!("CREATE DATABASE \"{safe}\""))
                        .execute(&maintenance_pool)
                        .await;
                }

                maintenance_pool.close().await;
                Ok::<(), anyhow::Error>(())
            })
            .await;
        init_result.context("ensuring the shared schema-per-test database exists")?;
        Ok(())
    }
}
