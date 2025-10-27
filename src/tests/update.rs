mod easy_lib {
    pub use crate as sql;
}

use super::Database;
use anyhow::Context;
use easy_lib::sql::{Insert, Output, Table, Update, sql};
use easy_macros::macros::always_context;
use sql_macros::sql_convenience;

#[derive(Table)]
#[sql(version = 1)]
#[sql(unique_id = "7b526dc4-30d4-4a67-8756-945a2f9c0004")]
struct ExampleTableIncrement {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    field: i64,
}

#[derive(Insert, Update, Output, Debug)]
#[sql(table = ExampleTableIncrement)]
#[sql(default = id)]
struct ExampleInsert {
    pub field: i64,
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_update_functionality() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;

    // Insert a row
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 5 }).await?;

    // Update the row
    ExampleTableIncrement::update(&mut conn, &ExampleInsert { field: 10 }, sql!(id = 1)).await?;

    // Select and verify
    let updated: ExampleInsert = ExampleTableIncrement::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(updated.field, 10);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_update_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 5 }).await?;
    // Update with no matching id
    ExampleTableIncrement::update(&mut conn, &ExampleInsert { field: 99 }, sql!(id = 999)).await?;
    let row: ExampleInsert = ExampleTableIncrement::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 5);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_update_multiple_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 1 }).await?;
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 2 }).await?;
    ExampleTableIncrement::update(&mut conn, &ExampleInsert { field: 42 }, sql!(id >= 1)).await?;
    let row1: ExampleInsert = ExampleTableIncrement::get(&mut conn, sql!(id = 1)).await?;
    let row2: ExampleInsert = ExampleTableIncrement::get(&mut conn, sql!(id = 2)).await?;
    assert_eq!(row1.field, 42);
    assert_eq!(row2.field, 42);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_update_sql_set_arithmetic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 10 }).await?;
    ExampleTableIncrement::update(&mut conn, sql!(field = field + 5), sql!(id = 1)).await?;
    let row: ExampleInsert = ExampleTableIncrement::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 15);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_update_rollback() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 7 }).await?;
    ExampleTableIncrement::update(&mut conn, &ExampleInsert { field: 99 }, sql!(id = 1)).await?;
    conn.rollback().await?;
    let db2 = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn2 = db2.transaction().await?;
    // Should be empty after rollback
    let result: Result<ExampleInsert, _> =
        ExampleTableIncrement::get(&mut conn2, sql!(id = 1)).await;
    assert!(result.is_err() || result.unwrap().field != 99);
    Ok(())
}
