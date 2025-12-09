use super::Database;
use crate::{Insert, Output, Table, Update, sql};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::{query, sql_convenience};

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
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_functionality() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;

    // Insert a row
    let insert_data = ExampleInsert { field: 5 };
    query!(conn, INSERT INTO ExampleTableIncrement VALUES {insert_data}).await?;

    // Update the row
    let update_data = ExampleInsert { field: 10 };
    query!(conn, UPDATE ExampleTableIncrement SET {update_data} WHERE id = 1).await?;

    // Select and verify
    let updated: ExampleInsert =
        query!(conn, SELECT ExampleInsert FROM ExampleTableIncrement WHERE id = 1).await?;
    assert_eq!(updated.field, 10);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    let insert_data = ExampleInsert { field: 5 };
    query!(conn, INSERT INTO ExampleTableIncrement VALUES {insert_data}).await?;
    // Update with no matching id
    let update_data = ExampleInsert { field: 99 };
    query!(conn, UPDATE ExampleTableIncrement SET {update_data} WHERE id = 999).await?;
    let row: ExampleInsert =
        query!(conn, SELECT ExampleInsert FROM ExampleTableIncrement WHERE id = 1).await?;
    assert_eq!(row.field, 5);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_multiple_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    let insert_data1 = ExampleInsert { field: 1 };
    query!(conn, INSERT INTO ExampleTableIncrement VALUES {insert_data1}).await?;
    let insert_data2 = ExampleInsert { field: 2 };
    query!(conn, INSERT INTO ExampleTableIncrement VALUES {insert_data2}).await?;
    let update_data = ExampleInsert { field: 42 };
    query!(conn, UPDATE ExampleTableIncrement SET {update_data} WHERE id >= 1).await?;
    let row1: ExampleInsert =
        query!(conn, SELECT ExampleInsert FROM ExampleTableIncrement WHERE id = 1).await?;
    let row2: ExampleInsert =
        query!(conn, SELECT ExampleInsert FROM ExampleTableIncrement WHERE id = 2).await?;
    assert_eq!(row1.field, 42);
    assert_eq!(row2.field, 42);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_sql_set_arithmetic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    let insert_data = ExampleInsert { field: 10 };
    query!(conn, INSERT INTO ExampleTableIncrement VALUES {insert_data}).await?;
    query!(conn, UPDATE ExampleTableIncrement SET field = field + 5 WHERE id = 1).await?;
    let row: ExampleInsert =
        query!(conn, SELECT ExampleInsert FROM ExampleTableIncrement WHERE id = 1).await?;
    assert_eq!(row.field, 15);
    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_rollback() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn = db.transaction().await?;
    let insert_data = ExampleInsert { field: 7 };
    query!(conn, INSERT INTO ExampleTableIncrement VALUES {insert_data}).await?;
    let update_data = ExampleInsert { field: 99 };
    query!(conn, UPDATE ExampleTableIncrement SET {update_data} WHERE id = 1).await?;
    conn.rollback().await?;
    let db2 = Database::setup_for_testing::<ExampleTableIncrement>().await?;
    let mut conn2 = db2.transaction().await?;
    // Should be empty after rollback
    let result: Result<ExampleInsert, _> =
        query!(conn2, SELECT ExampleInsert FROM ExampleTableIncrement WHERE id = 1).await;
    assert!(result.is_err() || result.unwrap().field != 99);
    Ok(())
}
