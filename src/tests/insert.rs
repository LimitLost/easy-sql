mod easy_lib {
    pub use crate as sql;
}

use anyhow::Context;
use easy_lib::sql::{SqlInsert, SqlOutput, SqlTable, SqlUpdate, sql};
use super::Database;
use easy_macros::macros::always_context;
use lazy_static::lazy_static;
use sql_macros::{sql_convenience};

lazy_static!{
    static ref ALL_CHARACTERS: String="1234567890abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!@#$%^&*()_+-=[]{}|;':\",.<>/?`~\\ ".to_string();
    
}

#[derive(SqlTable, Debug)]
#[sql(version = 1)]
#[sql(unique_id = "ce3d4f19-9d47-4fe2-9700-0957df4c04ee")]
struct ExampleTableInsert {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    field: i64,
    #[sql(default = ALL_CHARACTERS.clone())]
    field_str: String,
    field_opt: Option<i32>,
}

#[derive(SqlInsert, SqlUpdate, SqlOutput, Debug)]
#[sql(table = ExampleTableInsert)]
#[sql(default = id)]
struct ExampleInsert {
    pub field: i64,
    pub field_str: String,
    pub field_opt: Option<i32>,
}

#[derive(SqlInsert, Debug)]
#[sql(table = ExampleTableInsert)]
#[sql(default = id, field_str)]
struct ExampleInsertDefaultCheck {
    pub field: i64,
    pub field_opt: Option<i32>,
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsert {
            field: 1,
            field_str: "A".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let row: ExampleInsert = ExampleTableInsert::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 1);
    assert_eq!(row.field_str, "A");
    assert_eq!(row.field_opt, None);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_multiple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=5 {
        ExampleTableInsert::insert(
            &mut conn,
            &ExampleInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: Some(i as i32),
            },
        )
        .await?;
    }
    let rows: Vec<ExampleInsert> =
        ExampleTableInsert::select(&mut conn, sql!(id >= 1)).await?;
    assert_eq!(rows.len(), 5);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_default_value() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsertDefaultCheck {
            field: 2,
            field_opt: None,
        },
    )
    .await?;
    let row: ExampleInsert = ExampleTableInsert::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field_str, *ALL_CHARACTERS);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_and_rollback() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsert {
            field: 3,
            field_str: "X".to_string(),
            field_opt: None,
        },
    )
    .await?;
    conn.rollback().await?;
    let db2 = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn2 = db2.transaction().await?;
    let result: Result<ExampleTableInsert, anyhow::Error> =
        ExampleTableInsert::get(&mut conn2, sql!(id = 1)).await;
    assert!(result.is_err());
    Ok(())
}

#[always_context]
#[tokio::test]
async fn test_insert_duplicate_primary_key() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleTableInsert {
            id: 5,
            field: 4,
            field_str: "Y".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let result = ExampleTableInsert::insert(
        &mut conn,
        &ExampleTableInsert {
            id: 5,
            field: 4,
            field_str: "Y".to_string(),
            field_opt: None,
        },
    )
    .await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_nullable_field() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsert {
            field: 5,
            field_str: "Z".to_string(),
            field_opt: Some(123),
        },
    )
    .await?;
    let row: ExampleInsert = ExampleTableInsert::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field_opt, Some(123));
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_and_update() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsert {
            field: 6,
            field_str: "A".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableInsert::update(
        &mut conn,
        &ExampleInsert {
            field: 66,
            field_str: "B".to_string(),
            field_opt: Some(1),
        },
        sql!(id = 1),
    )
    .await?;
    let row: ExampleInsert = ExampleTableInsert::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 66);
    assert_eq!(row.field_str, "B");
    assert_eq!(row.field_opt, Some(1));
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_and_delete() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsert {
            field: 7,
            field_str: "C".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableInsert::delete(&mut conn, sql!(id = 1)).await?;
    let result: Result<ExampleTableInsert, anyhow::Error> =
        ExampleTableInsert::get(&mut conn, sql!(id = 1)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_boundary_values() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableInsert::insert(
        &mut conn,
        &ExampleInsert {
            field: i64::MAX,
            field_str: "MAX".to_string(),
            field_opt: Some(i32::MAX),
        },
    )
    .await?;
    let row: ExampleInsert = ExampleTableInsert::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, i64::MAX);
    assert_eq!(row.field_opt, Some(i32::MAX));
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_insert_select_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableInsert>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=3 {
        ExampleTableInsert::insert(
            &mut conn,
            &ExampleInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    let rows: Vec<ExampleInsert> =
        ExampleTableInsert::select(&mut conn, sql!(field >= 2)).await?;
    assert_eq!(rows.len(), 2);
    conn.rollback().await?;
    Ok(())
}
