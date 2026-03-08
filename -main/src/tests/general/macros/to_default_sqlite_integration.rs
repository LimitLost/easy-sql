use super::*;

use easy_macros::always_context;
use easy_sql_macros::query;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultSqliteCoreTable {
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
    #[sql(default = -8_i8)]
    default_i8: i8,
    #[sql(default = -16_i16)]
    default_i16: i16,
    #[sql(default = -32_i32)]
    default_i32: i32,
    #[sql(default = -64_i64)]
    default_i64: i64,
    #[sql(default = "sqlite-default".to_string())]
    default_string: String,
    #[sql(default = Some("sqlite-opt".to_string()))]
    default_option: Option<String>,
    #[sql(default = None::<String>)]
    default_option_none: Option<String>,
}

#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultSqliteCoreTable)]
#[sql(
    default = id,
    default_bool,
    default_f32,
    default_f64,
    default_i8,
    default_i16,
    default_i32,
    default_i64,
    default_string,
    default_option,
    default_option_none
)]
struct ToDefaultSqliteCoreInsert {
    marker: String,
}

#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultSqliteCoreTable)]
struct ToDefaultSqliteCoreRow {
    id: i32,
    marker: String,
    default_bool: bool,
    default_f32: f32,
    default_f64: f64,
    default_i8: i8,
    default_i16: i16,
    default_i32: i32,
    default_i64: i64,
    default_string: String,
    default_option: Option<String>,
    default_option_none: Option<String>,
}

#[cfg(feature = "chrono")]
fn sqlite_default_naive_date() -> chrono::NaiveDate {
    chrono::NaiveDate::from_ymd_opt(2024, 2, 3).unwrap()
}

#[cfg(feature = "chrono")]
fn sqlite_default_naive_datetime() -> chrono::NaiveDateTime {
    chrono::NaiveDateTime::new(
        sqlite_default_naive_date(),
        chrono::NaiveTime::from_hms_micro_opt(4, 5, 6, 123_000).unwrap(),
    )
}

#[cfg(feature = "chrono")]
fn sqlite_default_naive_time() -> chrono::NaiveTime {
    chrono::NaiveTime::from_hms_micro_opt(7, 8, 9, 456_000).unwrap()
}

#[cfg(feature = "chrono")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct ToDefaultSqliteChronoTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    marker: String,
    #[sql(default = sqlite_default_naive_date())]
    default_date: chrono::NaiveDate,
    #[sql(default = sqlite_default_naive_datetime())]
    default_datetime: chrono::NaiveDateTime,
    #[sql(default = sqlite_default_naive_time())]
    default_time: chrono::NaiveTime,
}

#[cfg(feature = "chrono")]
#[derive(Insert, Debug, Clone)]
#[sql(table = ToDefaultSqliteChronoTable)]
#[sql(default = id, default_date, default_datetime, default_time)]
struct ToDefaultSqliteChronoInsert {
    marker: String,
}

#[cfg(feature = "chrono")]
#[derive(Output, Debug, Clone)]
#[sql(table = ToDefaultSqliteChronoTable)]
struct ToDefaultSqliteChronoRow {
    id: i32,
    marker: String,
    default_date: chrono::NaiveDate,
    default_datetime: chrono::NaiveDateTime,
    default_time: chrono::NaiveTime,
}

#[always_context(skip(!))]
#[tokio::test]
async fn to_default_sqlite_integration_core_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultSqliteCoreTable>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultSqliteCoreInsert {
        marker: "sqlite-core".to_string(),
    };

    query!(pool, INSERT INTO ToDefaultSqliteCoreTable VALUES {insert}).await?;

    let row: ToDefaultSqliteCoreRow = query!(pool,
        SELECT ToDefaultSqliteCoreRow FROM ToDefaultSqliteCoreTable WHERE marker = "sqlite-core"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "sqlite-core");
    assert!(row.default_bool);
    assert!((row.default_f32 - 1.25_f32).abs() < f32::EPSILON);
    assert!((row.default_f64 - 2.5_f64).abs() < f64::EPSILON);
    assert_eq!(row.default_i8, -8_i8);
    assert_eq!(row.default_i16, -16_i16);
    assert_eq!(row.default_i32, -32_i32);
    assert_eq!(row.default_i64, -64_i64);
    assert_eq!(row.default_string, "sqlite-default");
    assert_eq!(row.default_option, Some("sqlite-opt".to_string()));
    assert_eq!(row.default_option_none, None);

    Ok(())
}

#[cfg(feature = "chrono")]
#[always_context(skip(!))]
#[tokio::test]
async fn to_default_sqlite_integration_chrono_defaults_roundtrip() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ToDefaultSqliteChronoTable>().await?;
    let mut pool = pool_resource.pool();

    let insert = ToDefaultSqliteChronoInsert {
        marker: "sqlite-chrono".to_string(),
    };

    query!(pool, INSERT INTO ToDefaultSqliteChronoTable VALUES {insert}).await?;

    let row: ToDefaultSqliteChronoRow = query!(pool,
        SELECT ToDefaultSqliteChronoRow FROM ToDefaultSqliteChronoTable WHERE marker = "sqlite-chrono"
    )
    .await?;

    assert_eq!(row.id, 1);
    assert_eq!(row.marker, "sqlite-chrono");
    assert_eq!(
        row.default_date,
        sqlite_default_naive_date()
    );
    assert_eq!(row.default_datetime, sqlite_default_naive_datetime());
    assert_eq!(row.default_time, sqlite_default_naive_time());

    Ok(())
}