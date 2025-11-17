use super::Database;
use crate::{Insert, Output, Table, Update, sql};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::sql_convenience;

#[derive(Table, Debug)]
#[sql(version = 1)]
#[sql(unique_id = "eee30e8b-ba04-4308-900c-066032ba5671")]
struct ExampleTableSelect {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    field: i64,
    field_str: String,
    field_opt: Option<i32>,
}

#[derive(Insert, Update, Output, Debug)]
#[sql(table = ExampleTableSelect)]
#[sql(default = id)]
struct ExampleSelectInsert {
    pub field: i64,
    pub field_str: String,
    pub field_opt: Option<i32>,
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: 1,
            field_str: "A".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(id = 1)).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field, 1);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(id = 999)).await?;
    assert_eq!(rows.len(), 0);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_multiple_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=5 {
        ExampleTableSelect::insert(
            &mut conn,
            &ExampleSelectInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(id >= 2 AND id <= 4)).await?;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].field, 2);
    assert_eq!(rows[2].field, 4);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_where_field() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: 10,
            field_str: "X".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(field = 10)).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field_str, "X");
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_all_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=3 {
        ExampleTableSelect::insert(
            &mut conn,
            &ExampleSelectInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    let rows: Vec<ExampleSelectInsert> = ExampleTableSelect::select(&mut conn, sql!(true)).await?;
    assert_eq!(rows.len(), 3);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_boundary_values() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: i64::MAX,
            field_str: "MAX".to_string(),
            field_opt: Some(i32::MAX),
        },
    )
    .await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(field = { i64::MAX })).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field_opt, Some(i32::MAX));
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_nullable_field() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: 8,
            field_str: "N".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(field_opt IS NULL)).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field, 8);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_after_update() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: 9,
            field_str: "U".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableSelect::update(
        &mut conn,
        &ExampleSelectInsert {
            field: 99,
            field_str: "UU".to_string(),
            field_opt: Some(1),
        },
        sql!(id = 1),
    )
    .await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(field = 99)).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].field_str, "UU");
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_after_delete() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: 10,
            field_str: "D".to_string(),
            field_opt: None,
        },
    )
    .await?;
    ExampleTableSelect::delete(&mut conn, sql!(id = 1)).await?;
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(id = 1)).await?;
    assert_eq!(rows.len(), 0);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_ordering() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    for i in (1..=3).rev() {
        ExampleTableSelect::insert(
            &mut conn,
            &ExampleSelectInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    let rows: Vec<ExampleSelectInsert> = ExampleTableSelect::select(
        &mut conn,
        #[context(no)]
        sql!(ORDER BY field ASC),
    )
    .await?;
    assert_eq!(rows[0].field, 1);
    assert_eq!(rows[2].field, 3);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=5 {
        ExampleTableSelect::insert(
            &mut conn,
            &ExampleSelectInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: None,
            },
        )
        .await?;
    }
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(LIMIT 2)).await?;
    assert_eq!(rows.len(), 2);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_get_single_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableSelect::insert(
        &mut conn,
        &ExampleSelectInsert {
            field: 20,
            field_str: "G".to_string(),
            field_opt: None,
        },
    )
    .await?;
    let row: ExampleSelectInsert = ExampleTableSelect::get(&mut conn, sql!(field = 20)).await?;
    assert_eq!(row.field_str, "G");
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_get_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    let result: Result<ExampleSelectInsert, _> =
        ExampleTableSelect::get(&mut conn, sql!(id = 999)).await;
    assert!(result.is_err());
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_select_complex_condition() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableSelect>().await?;
    let mut conn = db.transaction().await?;
    for i in 1..=5 {
        ExampleTableSelect::insert(
            &mut conn,
            &ExampleSelectInsert {
                field: i,
                field_str: format!("S{i}"),
                field_opt: if i % 2 == 0 { Some(i as i32) } else { None },
            },
        )
        .await?;
    }
    let rows: Vec<ExampleSelectInsert> =
        ExampleTableSelect::select(&mut conn, sql!(field_opt IS NOT NULL AND field >= 2)).await?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].field, 2);
    assert_eq!(rows[1].field, 4);
    conn.rollback().await?;
    Ok(())
}
