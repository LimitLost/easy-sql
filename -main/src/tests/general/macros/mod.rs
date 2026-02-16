// Test module for query! and query_lazy! macros

use super::{Database, TestDriver};
use crate::{Insert, Output, Table, Update};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
pub struct SqlxPoolTestResource {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

#[always_context(skip(!))]
#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
impl SqlxPoolTestResource {
    pub fn pool(&self) -> &sqlx::Pool<sqlx::Sqlite> {
        &self.pool
    }
}

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
#[always_context(skip(!))]
pub async fn setup_sqlx_pool_for_testing<T: crate::DatabaseSetup<TestDriver>>()
-> anyhow::Result<SqlxPoolTestResource> {
    use tokio::sync::Mutex;

    use crate::tests::init_test_logger;
    use crate::{Connection, EasySqlTables};
    use sqlx::sqlite::SqliteConnectOptions;

    init_test_logger();

    lazy_static::lazy_static! {
        static ref CURRENT_NAME_N:Mutex<usize>=Default::default();
    }
    let base_dir = std::env::temp_dir().join("easy_sql_pool_tests");
    std::fs::create_dir_all(&base_dir)?;
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("System time is before UNIX_EPOCH")?
        .as_nanos();
    let test_db_path = {
        let mut current_n = CURRENT_NAME_N.lock().await;
        let path = base_dir.join(format!("test_db_pool_{}_{}.sqlite", now_nanos, *current_n));
        *current_n += 1;
        path
    };

    let pool = sqlx::Pool::<sqlx::Sqlite>::connect_with(
        SqliteConnectOptions::default()
            .filename(&test_db_path)
            .create_if_missing(true),
    )
    .await?;

    let mut conn = Connection::new(pool.acquire().await?);

    <EasySqlTables as crate::DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;
    <T as crate::DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    Ok(SqlxPoolTestResource { pool })
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
pub struct SqlxPoolTestResource {
    pool: sqlx::Pool<sqlx::Postgres>,
    test_database: String,
    host: String,
    port: u16,
    username: String,
    password: String,
}

#[always_context(skip(!))]
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
impl SqlxPoolTestResource {
    pub fn pool(&self) -> &sqlx::Pool<sqlx::Postgres> {
        &self.pool
    }

    async fn cleanup_inner(
        pool: sqlx::Pool<sqlx::Postgres>,
        test_database: String,
        host: String,
        port: u16,
        username: String,
        password: String,
    ) -> anyhow::Result<()> {
        use sqlx::postgres::PgConnectOptions;

        pool.close().await;

        let maintenance_pool = sqlx::Pool::<sqlx::Postgres>::connect_with(
            PgConnectOptions::new()
                .host(&host)
                .port(port)
                .username(&username)
                .password(&password),
        )
        .await?;

        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS \"{}\"",
            test_database.replace('"', "\"\"")
        ))
        .execute(&maintenance_pool)
        .await?;

        maintenance_pool.close().await;
        Ok(())
    }

    pub async fn cleanup(&self) -> anyhow::Result<()> {
        Self::cleanup_inner(
            self.pool.clone(),
            self.test_database.clone(),
            self.host.clone(),
            self.port,
            self.username.clone(),
            self.password.clone(),
        )
        .await
    }
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
impl Drop for SqlxPoolTestResource {
    fn drop(&mut self) {
        let pool = self.pool.clone();
        let test_database = self.test_database.clone();
        let host = self.host.clone();
        let port = self.port;
        let username = self.username.clone();
        let password = self.password.clone();

        let _ = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(runtime) = runtime {
                let _ = runtime.block_on(SqlxPoolTestResource::cleanup_inner(
                    pool,
                    test_database,
                    host,
                    port,
                    username,
                    password,
                ));
            }
        })
        .join();
    }
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
#[always_context(skip(!))]
pub async fn setup_sqlx_pool_for_testing<T: crate::DatabaseSetup<TestDriver>>()
-> anyhow::Result<SqlxPoolTestResource> {
    use tokio::sync::Mutex;

    use crate::tests::init_test_logger;
    use crate::{Connection, EasySqlTables};
    use sqlx::postgres::PgConnectOptions;

    init_test_logger();

    lazy_static::lazy_static! {
        static ref CURRENT_NAME_N:Mutex<usize>=Default::default();
    }

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
        let name = format!("{}_pool_{}", db_prefix, *current_n);
        *current_n += 1;
        name
    };

    let maintenance_pool = sqlx::Pool::<sqlx::Postgres>::connect_with(
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

    let pool = sqlx::Pool::<sqlx::Postgres>::connect_with(
        PgConnectOptions::new()
            .host(&host)
            .port(port)
            .username(&username)
            .password(&password)
            .database(&test_database),
    )
    .await?;

    let mut conn = Connection::new(pool.acquire().await?);

    <EasySqlTables as crate::DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;
    <T as crate::DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    Ok(SqlxPoolTestResource {
        pool,
        test_database,
        host,
        port,
        username,
        password,
    })
}

// ====================
// Shared Test Tables
// ====================

/// Main test table for SQL expression testing
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct ExprTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub int_field: i32,
    pub str_field: String,
    pub bool_field: bool,
    pub nullable_field: Option<String>,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = ExprTestTable)]
#[sql(default = id)]
pub struct ExprTestData {
    pub int_field: i32,
    pub str_field: String,
    pub bool_field: bool,
    pub nullable_field: Option<String>,
}

/// Table for testing relationships and joins
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct RelatedTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    #[sql(foreign_key = ExprTestTable)]
    pub parent_id: i32,
    pub data: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = RelatedTestTable)]
#[sql(default = id)]
pub struct RelatedTestData {
    pub parent_id: i32,
    pub data: String,
}

/// Table for testing numeric operations
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct NumericTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub int_val: i32,
    pub float_val: Option<f64>,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = NumericTestTable)]
#[sql(default = id)]
pub struct NumericTestData {
    pub int_val: i32,
    pub float_val: Option<f64>,
}

// ====================
// Test Utilities
// ====================

/// Helper to create test data with default values
pub fn default_expr_test_data() -> ExprTestData {
    ExprTestData {
        int_field: 42,
        str_field: "test".to_string(),
        bool_field: true,
        nullable_field: None,
    }
}

/// Helper to create test data with custom values
pub fn expr_test_data(
    int_field: i32,
    str_field: &str,
    bool_field: bool,
    nullable_field: Option<&str>,
) -> ExprTestData {
    ExprTestData {
        int_field,
        str_field: str_field.to_string(),
        bool_field,
        nullable_field: nullable_field.map(|s| s.to_string()),
    }
}

/// Helper to insert test data and return its ID
#[always_context(skip(!))]
pub async fn insert_test_data(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    data: ExprTestData,
) -> anyhow::Result<()> {
    query!(conn, INSERT INTO ExprTestTable VALUES {data})
        .await
        .context("Failed to insert test data")?;
    Ok(())
}

/// Helper to insert multiple test records
#[always_context(skip(!))]
pub async fn insert_multiple_test_data(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    data_vec: Vec<ExprTestData>,
) -> anyhow::Result<()> {
    query!(conn, INSERT INTO ExprTestTable VALUES {data_vec}).await?;
    Ok(())
}

// ====================
// Sub-modules
// ====================

mod custom_select;
mod custom_select_compile_fail;
mod order_by_container_test;
mod order_by_output_columns_test;
mod output_columns_comprehensive_test;
mod output_columns_in_custom_select_test;
mod pool_argument_test;
mod query_lazy_macro;
mod query_macro;
mod sql_expressions;

mod custom_select_validation_test;
mod custom_sql_functions;
mod generic_easy_executor_argument_test;
mod sql_case_insensitive_functions;
mod sql_functions;

mod table_join;
