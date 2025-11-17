use super::Database;
use crate::{Insert, Output, Table, Update, sql};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::sql_convenience;

#[derive(Table, Debug)]
#[sql(version = 1)]
#[sql(unique_id = "ff126623-4355-449a-921c-31e65f3449e8")]
struct ExampleTableDelete {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    field: i64,
    field_str: String,
    field_opt: Option<i32>,
}

#[derive(Insert, Update, Output, Debug)]
#[sql(table = ExampleTableDelete)]
#[sql(default = id)]
struct ExampleDeleteInsert {
    pub field: i64,
    pub field_str: String,
    pub field_opt: Option<i32>,
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: 1,
            field_str: "A".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(id = 1)).await?;
    let result: Result<ExampleDeleteInsert, _> =
        ExampleTableDelete::get(&mut conn, sql!(id = 1)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: 2,
            field_str: "B".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(id = 999)).await?;
    let row: ExampleDeleteInsert = ExampleTableDelete::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 2);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_multiple_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=3 {
        ExampleTableDelete::insert(
            &mut conn,
            &ExampleDeleteInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    ExampleTableDelete::delete(&mut conn, sql!(id >= 2)).await?;
    let rows: Vec<ExampleDeleteInsert> =
        ExampleTableDelete::select(&mut conn, sql!(id >= 1)).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field, 1);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_where_field() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: 10,
            field_str: "X".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(field = 10)).await?;
    let result: Result<ExampleDeleteInsert, _> =
        ExampleTableDelete::get(&mut conn, sql!(id = 1)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_all_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=3 {
        ExampleTableDelete::insert(
            &mut conn,
            &ExampleDeleteInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    ExampleTableDelete::delete(&mut conn, sql!(true)).await?;
    let rows: Vec<ExampleDeleteInsert> = ExampleTableDelete::select(&mut conn, sql!(true)).await?;
    assert_eq!(rows.len(), 0);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_and_rollback() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: 4,
            field_str: "Y".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(id = 1)).await?;
    conn.rollback().await?;
    let db2 = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn2 = db2.transaction().await?;
    let result: Result<ExampleDeleteInsert, _> =
        ExampleTableDelete::get(&mut conn2, sql!(id = 1)).await;
    assert!(result.is_err());
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_boundary_values() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: i64::MAX,
            field_str: "MAX".to_string(),
            field_opt: Some(i32::MAX),
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(field = { i64::MAX })).await?;
    let result: Result<ExampleDeleteInsert, _> =
        ExampleTableDelete::get(&mut conn, sql!(id = 1)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_nullable_field() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: 8,
            field_str: "N".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(field_opt IS NULL)).await?;
    let result: Result<ExampleDeleteInsert, _> =
        ExampleTableDelete::get(&mut conn, sql!(id = 1)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_after_update() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleDeleteInsert {
            field: 9,
            field_str: "U".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::update(
        &mut conn,
        &ExampleDeleteInsert {
            field: 99,
            field_str: "UU".to_string(),
            field_opt: Some(1),
        },
        sql!(id = 1),
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(field = 99)).await?;
    let result: Result<ExampleDeleteInsert, _> =
        ExampleTableDelete::get(&mut conn, sql!(id = 1)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_delete_and_reinsert() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableDelete>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleTableDelete {
            id: 1,
            field: 10,
            field_str: "R".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableDelete::delete(&mut conn, sql!(id = 1)).await?;
    ExampleTableDelete::insert(
        &mut conn,
        &ExampleTableDelete {
            id: 2,
            field: 11,
            field_str: "RR".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let row: ExampleDeleteInsert = ExampleTableDelete::get(&mut conn, sql!(id = 2)).await?;
    assert_eq!(row.field, 11);
    assert_eq!(row.field_str, "RR");
    conn.rollback().await?;
    Ok(())
}
