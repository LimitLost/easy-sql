use std::str::FromStr;

use super::*;
use easy_macros::always_context;
use easy_sql_macros::query;
use sqlx::postgres::types::{Oid, PgCiText, PgInterval, PgLQuery, PgLTree};
#[cfg(feature = "ipnet")]
use std::net::IpAddr;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresCoreTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = true)]
    default_bool: bool,
    #[sql(default = 1.25_f32)]
    default_f32: f32,
    #[sql(default = 2.5_f64)]
    default_f64: f64,
    #[sql(default = -16_i16)]
    default_i16: i16,
    #[sql(default = -32_i32)]
    default_i32: i32,
    #[sql(default = -64_i64)]
    default_i64: i64,
    #[sql(default = "postgres-default".to_string())]
    default_string: String,
    #[sql(default = PgInterval {
        months: 2,
        days: 3,
        microseconds: 4_000_000
    })]
    default_pg_interval: PgInterval,
    #[sql(default = Oid(42))]
    default_oid: Oid,
    #[sql(default = PgCiText("Hello Citext".to_string()))]
    default_citext: PgCiText,
    #[sql(default = PgLQuery::from_str("Top.Science").unwrap())]
    default_lquery: PgLQuery,
    #[sql(default = PgLTree::from_str("Top.Science").unwrap())]
    default_ltree: PgLTree,
    #[sql(default = vec![0_u8, 1_u8, 2_u8, 255_u8])]
    default_bytes: Vec<u8>,
    #[sql(default = Some("postgres-opt".to_string()))]
    default_option: Option<String>,
    #[sql(default = None::<String>)]
    default_option_none: Option<String>,
}

#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresCoreTable)]
#[sql(
    default = id,
    default_bool,
    default_f32,
    default_f64,
    default_i16,
    default_i32,
    default_i64,
    default_string,
    default_pg_interval,
    default_oid,
    default_citext,
    default_lquery,
    default_ltree,
    default_bytes,
    default_option,
    default_option_none
)]
struct ToDefaultPostgresCoreInsert {
    marker: String,
}

#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresCoreTable)]
struct ToDefaultPostgresCoreRow {
    id: i32,
    marker: String,
    default_bool: bool,
    default_f32: f32,
    default_f64: f64,
    default_i16: i16,
    default_i32: i32,
    default_i64: i64,
    default_string: String,
    default_pg_interval: PgInterval,
    default_oid: Oid,
    default_citext: PgCiText,
    default_lquery: PgLQuery,
    default_ltree: PgLTree,
    default_bytes: Vec<u8>,
    default_option: Option<String>,
    default_option_none: Option<String>,
}

#[cfg(feature = "ipnet")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresIpNetTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = sqlx::types::ipnet::IpNet::from_str("10.1.0.0/16").unwrap())]
    default_ipnet: sqlx::types::ipnet::IpNet,
    #[sql(default = IpAddr::from_str("10.1.2.3").unwrap())]
    default_ipaddr: IpAddr,
}

#[cfg(feature = "ipnet")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresIpNetTable)]
struct ToDefaultPostgresIpNetRow {
    id: i32,
    marker: String,
    default_ipnet: sqlx::types::ipnet::IpNet,
    default_ipaddr: IpAddr,
}

#[cfg(feature = "ipnet")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresIpNetTable)]
#[sql(default = id, default_ipnet, default_ipaddr)]
struct ToDefaultPostgresIpNetInsert {
    marker: String,
}

#[cfg(feature = "json")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresJsonTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = serde_json::json!({"kind":"json-default","n":7}))]
    default_json: sqlx::types::JsonValue,
}

#[cfg(feature = "json")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresJsonTable)]
#[sql(default = id, default_json)]
struct ToDefaultPostgresJsonInsert {
    marker: String,
}

#[cfg(feature = "json")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresJsonTable)]
struct ToDefaultPostgresJsonRow {
    id: i32,
    marker: String,
    default_json: sqlx::types::JsonValue,
}

#[cfg(feature = "time")]
fn postgres_time_default_date() -> sqlx::types::time::Date {
    sqlx::types::time::Date::from_ordinal_date(2024, 2).unwrap()
}

#[cfg(feature = "time")]
fn postgres_time_default_time() -> sqlx::types::time::Time {
    sqlx::types::time::Time::from_hms_micro(3, 4, 5, 600_000).unwrap()
}

#[cfg(feature = "time")]
fn postgres_time_default_primitive_datetime() -> sqlx::types::time::PrimitiveDateTime {
    sqlx::types::time::PrimitiveDateTime::new(
        postgres_time_default_date(),
        postgres_time_default_time(),
    )
}

#[cfg(feature = "time")]
fn postgres_time_default_offset_datetime() -> sqlx::types::time::OffsetDateTime {
    sqlx::types::time::OffsetDateTime::from_unix_timestamp(0).unwrap()
}

#[cfg(feature = "time")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresTimeTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = postgres_time_default_date())]
    default_date: sqlx::types::time::Date,
    #[sql(default = postgres_time_default_time())]
    default_time: sqlx::types::time::Time,
    #[sql(default = postgres_time_default_primitive_datetime())]
    default_primitive_datetime: sqlx::types::time::PrimitiveDateTime,
    #[sql(default = postgres_time_default_offset_datetime())]
    default_offset_datetime: sqlx::types::time::OffsetDateTime,
}

#[cfg(feature = "time")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresTimeTable)]
struct ToDefaultPostgresTimeRow {
    id: i32,
    marker: String,
    default_date: sqlx::types::time::Date,
    default_time: sqlx::types::time::Time,
    default_primitive_datetime: sqlx::types::time::PrimitiveDateTime,
    default_offset_datetime: sqlx::types::time::OffsetDateTime,
}

#[cfg(feature = "time")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresTimeTable)]
#[sql(
    default =
        id,
        default_date,
        default_time,
        default_primitive_datetime,
        default_offset_datetime
)]
struct ToDefaultPostgresTimeInsert {
    marker: String,
}

#[cfg(feature = "bigdecimal")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresBigDecimalTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = bigdecimal::BigDecimal::from_str("12345.6789").unwrap())]
    default_bigdecimal: bigdecimal::BigDecimal,
}

#[cfg(feature = "bigdecimal")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresBigDecimalTable)]
#[sql(default = id, default_bigdecimal)]
struct ToDefaultPostgresBigDecimalInsert {
    marker: String,
}

#[cfg(feature = "bigdecimal")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresBigDecimalTable)]
struct ToDefaultPostgresBigDecimalRow {
    id: i32,
    marker: String,
    default_bigdecimal: bigdecimal::BigDecimal,
}

#[cfg(feature = "rust_decimal")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresRustDecimalTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = rust_decimal::Decimal::from_str("9876.5432").unwrap())]
    default_rust_decimal: rust_decimal::Decimal,
}

#[cfg(feature = "rust_decimal")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresRustDecimalTable)]
#[sql(default = id, default_rust_decimal)]
struct ToDefaultPostgresRustDecimalInsert {
    marker: String,
}

#[cfg(feature = "rust_decimal")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresRustDecimalTable)]
struct ToDefaultPostgresRustDecimalRow {
    id: i32,
    marker: String,
    default_rust_decimal: rust_decimal::Decimal,
}

#[cfg(feature = "chrono")]
fn postgres_chrono_default_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2024, 2, 3).unwrap()
}

#[cfg(feature = "chrono")]
fn postgres_chrono_default_datetime() -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::new(
        postgres_chrono_default_date(),
        chrono::NaiveTime::from_hms_micro_opt(4, 5, 6, 789_000).unwrap(),
    )
}

#[cfg(feature = "chrono")]
fn postgres_chrono_default_time() -> chrono::NaiveTime {
    chrono::NaiveTime::from_hms_micro_opt(7, 8, 9, 123_000).unwrap()
}

#[cfg(feature = "chrono")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresChronoTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = postgres_chrono_default_date())]
    default_date: chrono::NaiveDate,
    #[sql(default = postgres_chrono_default_datetime())]
    default_datetime: chrono::NaiveDateTime,
    #[sql(default = postgres_chrono_default_time())]
    default_time: chrono::NaiveTime,
}

#[cfg(feature = "chrono")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresChronoTable)]
struct ToDefaultPostgresChronoRow {
    id: i32,
    marker: String,
    default_date: chrono::NaiveDate,
    default_datetime: chrono::NaiveDateTime,
    default_time: chrono::NaiveTime,
}

#[cfg(feature = "chrono")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresChronoTable)]
#[sql(default = id, default_date, default_datetime, default_time)]
struct ToDefaultPostgresChronoInsert {
    marker: String,
}

#[cfg(feature = "uuid")]
fn postgres_uuid_default_value() -> uuid::Uuid {
    uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()
}

#[cfg(feature = "uuid")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultPostgresUuidTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = postgres_uuid_default_value())]
    default_uuid: uuid::Uuid,
}

#[cfg(feature = "uuid")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultPostgresUuidTable)]
#[sql(default = id, default_uuid)]
struct ToDefaultPostgresUuidInsert {
    marker: String,
}

#[cfg(feature = "uuid")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultPostgresUuidTable)]
struct ToDefaultPostgresUuidRow {
    id: i32,
    marker: String,
    default_uuid: uuid::Uuid,
}

struct ToDefaultPostgresSetup;

#[always_context(skip(!))]
impl crate::DatabaseSetup<TestDriver> for ToDefaultPostgresSetup {
    #[no_context_inputs]
    async fn setup(
        conn: &mut (impl crate::EasyExecutor<TestDriver> + Send + Sync),
    ) -> anyhow::Result<()> {
        sqlx::query("CREATE EXTENSION IF NOT EXISTS citext")
            .execute(conn.executor())
            .await?;
        sqlx::query("CREATE EXTENSION IF NOT EXISTS ltree")
            .execute(conn.executor())
            .await?;

        <ToDefaultPostgresCoreTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn).await?;

        #[cfg(feature = "ipnet")]
        <ToDefaultPostgresIpNetTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn)
            .await?;
        #[cfg(feature = "json")]
        <ToDefaultPostgresJsonTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn).await?;
        #[cfg(feature = "time")]
        <ToDefaultPostgresTimeTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn).await?;
        #[cfg(feature = "bigdecimal")]
        <ToDefaultPostgresBigDecimalTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn)
            .await?;
        #[cfg(feature = "rust_decimal")]
        <ToDefaultPostgresRustDecimalTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn)
            .await?;
        #[cfg(feature = "chrono")]
        <ToDefaultPostgresChronoTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn)
            .await?;
        #[cfg(feature = "uuid")]
        <ToDefaultPostgresUuidTable as crate::DatabaseSetup<TestDriver>>::setup(&mut *conn).await?;

        Ok(())
    }
}

#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_core_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresCoreInsert {
        marker: "postgres-core".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresCoreTable VALUES {insert}).await?;

    let row: ToDefaultPostgresCoreRow = query!(pool,
        SELECT ToDefaultPostgresCoreRow FROM ToDefaultPostgresCoreTable WHERE marker = "postgres-core"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-core");
    assert!(row.default_bool);
    assert!((row.default_f32 - 1.25_f32).abs() < f32::EPSILON);
    assert!((row.default_f64 - 2.5_f64).abs() < f64::EPSILON);
    assert_eq!(row.default_i16, -16_i16);
    assert_eq!(row.default_i32, -32_i32);
    assert_eq!(row.default_i64, -64_i64);
    assert_eq!(row.default_string, "postgres-default");
    assert_eq!(row.default_pg_interval.months, 2);
    assert_eq!(row.default_pg_interval.days, 3);
    assert_eq!(row.default_pg_interval.microseconds, 4_000_000);
    assert_eq!(row.default_oid.0, 42);
    assert_eq!(row.default_citext.to_string(), "Hello Citext");
    assert_eq!(row.default_lquery.to_string(), "Top.Science");
    assert_eq!(row.default_ltree.to_string(), "Top.Science");
    assert_eq!(row.default_bytes, vec![0_u8, 1_u8, 2_u8, 255_u8]);
    assert_eq!(row.default_option, Some("postgres-opt".to_string()));
    assert_eq!(row.default_option_none, None);

    Ok(())
}

#[cfg(feature = "ipnet")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_ipnet_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresIpNetInsert {
        marker: "postgres-ipnet".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresIpNetTable VALUES {insert}).await?;

    let row: ToDefaultPostgresIpNetRow = query!(pool,
        SELECT ToDefaultPostgresIpNetRow FROM ToDefaultPostgresIpNetTable WHERE marker = "postgres-ipnet"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-ipnet");
    assert_eq!(row.default_ipnet.to_string(), "10.1.0.0/16");
    assert_eq!(row.default_ipaddr.to_string(), "10.1.2.3");

    Ok(())
}

#[cfg(feature = "json")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_json_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresJsonInsert {
        marker: "postgres-json".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresJsonTable VALUES {insert}).await?;

    let row: ToDefaultPostgresJsonRow = query!(pool,
        SELECT ToDefaultPostgresJsonRow FROM ToDefaultPostgresJsonTable WHERE marker = "postgres-json"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-json");
    assert_eq!(
        row.default_json,
        serde_json::json!({"kind":"json-default","n":7})
    );

    Ok(())
}

#[cfg(feature = "time")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_time_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresTimeInsert {
        marker: "postgres-time".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresTimeTable VALUES {insert}).await?;

    let row: ToDefaultPostgresTimeRow = query!(pool,
        SELECT ToDefaultPostgresTimeRow FROM ToDefaultPostgresTimeTable WHERE marker = "postgres-time"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-time");
    assert_eq!(row.default_date.to_string(), "2024-01-02");
    assert!(matches!(
        row.default_time.to_string().as_str(),
        "03:04:05.6" | "3:04:05.6"
    ));
    assert!(matches!(
        row.default_primitive_datetime.to_string().as_str(),
        "2024-01-02 03:04:05.6" | "2024-01-02 3:04:05.6"
    ));
    assert_eq!(row.default_offset_datetime.unix_timestamp(), 0);

    Ok(())
}

#[cfg(feature = "bigdecimal")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_bigdecimal_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresBigDecimalInsert {
        marker: "postgres-bigdecimal".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresBigDecimalTable VALUES {insert}).await?;

    let row: ToDefaultPostgresBigDecimalRow = query!(pool,
        SELECT ToDefaultPostgresBigDecimalRow FROM ToDefaultPostgresBigDecimalTable WHERE marker = "postgres-bigdecimal"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-bigdecimal");
    assert_eq!(row.default_bigdecimal.to_string(), "12345.6789");

    Ok(())
}

#[cfg(feature = "rust_decimal")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_rust_decimal_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresRustDecimalInsert {
        marker: "postgres-rust-decimal".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresRustDecimalTable VALUES {insert}).await?;

    let row: ToDefaultPostgresRustDecimalRow = query!(pool,
        SELECT ToDefaultPostgresRustDecimalRow FROM ToDefaultPostgresRustDecimalTable WHERE marker = "postgres-rust-decimal"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-rust-decimal");
    assert_eq!(row.default_rust_decimal.to_string(), "9876.5432");

    Ok(())
}

#[cfg(feature = "chrono")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_chrono_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresChronoInsert {
        marker: "postgres-chrono".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresChronoTable VALUES {insert}).await?;

    let row: ToDefaultPostgresChronoRow = query!(pool,
        SELECT ToDefaultPostgresChronoRow FROM ToDefaultPostgresChronoTable WHERE marker = "postgres-chrono"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-chrono");
    assert_eq!(
        row.default_date,
        chrono::NaiveDate::from_ymd_opt(2024, 2, 3).unwrap()
    );
    assert_eq!(
        row.default_datetime,
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(2024, 2, 3).unwrap(),
            chrono::NaiveTime::from_hms_micro_opt(4, 5, 6, 789_000).unwrap()
        )
    );
    assert_eq!(
        row.default_time,
        chrono::NaiveTime::from_hms_micro_opt(7, 8, 9, 123_000).unwrap()
    );

    Ok(())
}

#[cfg(feature = "uuid")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_postgres_integration_uuid_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultPostgresSetup>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultPostgresUuidInsert {
        marker: "postgres-uuid".to_string(),
    };
    query!(pool, INSERT INTO ToDefaultPostgresUuidTable VALUES {insert}).await?;

    let row: ToDefaultPostgresUuidRow = query!(pool,
        SELECT ToDefaultPostgresUuidRow FROM ToDefaultPostgresUuidTable WHERE marker = "postgres-uuid"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "postgres-uuid");
    assert_eq!(
        row.default_uuid,
        uuid::Uuid::parse_str("123e4567-e89b-12d3-a456-426614174000").unwrap()
    );

    Ok(())
}
