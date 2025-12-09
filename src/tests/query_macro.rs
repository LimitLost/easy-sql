//! Tests for the query! macro

use crate::{Insert, Output, Table, Update};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::{query, sql, sql_convenience};

use super::Database;

#[derive(Table, Debug)]
#[sql(version = 1)]
#[sql(unique_id = "865c36d9-2998-46bc-bfd5-9769b6ad715b")]
struct QueryTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i64,
    field1: String,
    field2: i32,
    field3: i64,
}

#[derive(Insert, Update, Output, Debug)]
#[sql(table = QueryTestTable)]
#[sql(default = id)]
struct QueryTestData {
    pub field1: String,
    pub field2: i32,
    pub field3: i64,
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "test".to_string(),
            field2: 42,
            field3: 100,
        }}
    )
    .await?;

    // Test SELECT with WHERE clause
    let random_id = 1i64;
    let result =
        query!(conn, SELECT QueryTestData FROM QueryTestTable WHERE id = {random_id}).await?;

    assert_eq!(result.field1, "test");
    assert_eq!(result.field2, 42);
    assert_eq!(result.field3, 100);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_optional_some() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "test".to_string(),
            field2: 42,
            field3: 100,
        }}
    )
    .await?;

    // Test SELECT with WHERE clause
    let random_id = 1i64;
    let result =
        query!(conn, SELECT Option<QueryTestData> FROM QueryTestTable WHERE id = {random_id})
            .await?;

    let result = result.context("Expected Some(result) but got None")?;

    assert_eq!(result.field1, "test");
    assert_eq!(result.field2, 42);
    assert_eq!(result.field3, 100);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_optional_none() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Ensure no data is present
    // Test SELECT with WHERE clause for non-existent ID
    let random_id = 999i64;
    let result =
        query!(conn, SELECT Option<QueryTestData> FROM QueryTestTable WHERE id = {random_id})
            .await?;

    assert!(result.is_none(), "Expected None but got Some(result)");

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_vec() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "test".to_string(),
            field2: 42,
            field3: 100,
        }}
    )
    .await?;

    // Test SELECT with WHERE clause
    let result =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE field2 = {42}).await?;

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].field1, "test");
    assert_eq!(result[0].field2, 42);
    assert_eq!(result[0].field3, 100);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    let new_data = QueryTestData {
        field1: "inserted".to_string(),
        field2: 99,
        field3: 200,
    };

    // Test INSERT
    query!(conn, INSERT INTO QueryTestTable VALUES {new_data}).await?;

    // Verify the insert
    let result: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE field1 = "inserted")
            .await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].field2, 99);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "to_delete".to_string(),
            field2: 5,
            field3: 10,
        }}
    )
    .await?;

    let delete_id = 1i64;

    // Test DELETE
    query!(conn, DELETE FROM QueryTestTable WHERE id = {delete_id}).await?;

    // Verify deletion
    let results: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE id = {delete_id}).await?;
    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Test EXISTS when no data
    let check_id = 999i64;
    let exists: bool = query!(conn, EXISTS QueryTestTable WHERE id = {check_id}).await?;
    assert!(!exists);

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "exists_test".to_string(),
            field2: 1,
            field3: 2,
        }}
    )
    .await?;

    // Test EXISTS when data exists
    let check_id = 1i64;
    let exists: bool = query!(conn, EXISTS QueryTestTable WHERE id = {check_id}).await?;
    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

// Additional comprehensive tests

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "test".to_string(),
            field2: 42,
            field3: 100,
        }}
    )
    .await?;

    let limit_val = 1;
    let result: QueryTestData = query!(
        conn,
        SELECT QueryTestData FROM QueryTestTable LIMIT {limit_val}
    )
    .await?;

    assert_eq!(result.field1, "test");

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    let new_data = QueryTestData {
        field1: "returned".to_string(),
        field2: 777,
        field3: 888,
    };

    // Test INSERT with RETURNING (temporarily disabled - type errors)

    let returned: QueryTestData = query!(
        conn,
        INSERT INTO QueryTestTable VALUES {new_data} RETURNING QueryTestData
    )
    .await?;

    assert_eq!(returned.field1, "returned");
    assert_eq!(returned.field2, 777);
    assert_eq!(returned.field3, 888);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_by_value() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "original".to_string(),
            field2: 10,
            field3: 20,
        }}
    )
    .await?;

    let update_id = 1i64;
    let new_value = "updated_by_value".to_string();
    let new_field2 = 88i32;

    // Test UPDATE by value
    let update_data = QueryTestData {
        field1: new_value,
        field2: new_field2,
        field3: 30,
    };

    query!(conn, UPDATE QueryTestTable SET {update_data} WHERE id = {update_id}).await?;

    // Verify update
    let results: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE id = {update_id}).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].field1, "updated_by_value");
    assert_eq!(results[0].field2, 88);
    assert_eq!(results[0].field3, 30);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    let update_id = 1i64;
    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestTable {
            id: update_id,
            field1: "before".to_string(),
            field2: 10,
            field3: 20,
        }}
    )
    .await?;

    let update_data = QueryTestData {
        field1: "after".to_string(),
        field2: 30,
        field3: 40,
    };

    // First, verify the row was inserted
    let verify: QueryTestData =
        query!(conn, SELECT QueryTestData FROM QueryTestTable WHERE id = {update_id})
            .await
            .context("Failed to find inserted row")?;

    eprintln!(
        "DEBUG: Found row before update: field1={}, field2={}, field3={}",
        verify.field1, verify.field2, verify.field3
    );

    // Test UPDATE with RETURNING (temporarily disabled - type errors)

    eprintln!("DEBUG: About to execute UPDATE with RETURNING");

    let returned = /* query!(
        conn,
        UPDATE QueryTestTable SET {&update_data} WHERE id = {update_id} RETURNING Option<QueryTestData>
    ) */
   query!(
        conn,
        UPDATE QueryTestTable SET field1 = {update_data.field1}, field2 = {update_data.field2}, field3 = {update_data.field3} WHERE id = {update_id} RETURNING Option<QueryTestData>
    )
    .await?;

    // First, verify the row was updated
    let verify: QueryTestData =
        query!(conn, SELECT QueryTestData FROM QueryTestTable WHERE id = {update_id}).await?;
    eprintln!(
        "DEBUG: Found row after update: field1={}, field2={}, field3={}",
        verify.field1, verify.field2, verify.field3
    );
    assert_eq!(verify.field1, "after");
    assert_eq!(verify.field2, 30);
    assert_eq!(verify.field3, 40);

    if let Some(returned) = returned {
        assert_eq!(returned.field1, "after");
        assert_eq!(returned.field2, 30);
        assert_eq!(returned.field3, 40);
    } else {
        anyhow::bail!("Expected Some(returned) but got None");
    }

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "to_be_deleted".to_string(),
            field2: 111,
            field3: 222,
        }}
    )
    .await?;

    let delete_id = 1i64;

    // Test DELETE with RETURNING (temporarily disabled - type errors)

    let returned: QueryTestData = query!(
        conn,
        DELETE FROM QueryTestTable WHERE id = {delete_id} RETURNING QueryTestData
    )
    .await?;

    assert_eq!(returned.field1, "to_be_deleted");
    assert_eq!(returned.field2, 111);
    assert_eq!(returned.field3, 222);

    // Test DELETE without RETURNING
    query!(conn, DELETE FROM QueryTestTable WHERE id = {delete_id}).await?;

    // Verify deletion
    let results: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE id = {delete_id}).await?;
    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_without_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Test EXISTS when table is empty
    let exists: bool = query!(conn, EXISTS QueryTestTable).await.context("")?;
    assert!(!exists);

    // Insert test data
    QueryTestTable::insert(
        &mut conn,
        &QueryTestData {
            field1: "exists".to_string(),
            field2: 1,
            field3: 2,
        },
    )
    .await?;

    // Test EXISTS when table has data
    let exists: bool = query!(conn, EXISTS QueryTestTable).await.context("")?;
    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_complex_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "complex".to_string(),
            field2: 50,
            field3: 150,
        }}
    )
    .await?;

    let min_val = 40i32;
    let max_val = 60i32;
    let result: QueryTestData = query!(
        conn,
        SELECT QueryTestData FROM QueryTestTable WHERE field2 > {min_val} AND field2 < {max_val}
    )
    .await?;

    assert_eq!(result.field1, "complex");
    assert_eq!(result.field2, 50);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_distinct() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "distinct_test".to_string(),
            field2: 100,
            field3: 200,
        }}
    )
    .await?;

    let check_val = 100i32;
    let result: QueryTestData = query!(
        conn,
        SELECT DISTINCT QueryTestData FROM QueryTestTable WHERE field2 = {check_val}
    )
    .await?;

    assert_eq!(result.field1, "distinct_test");

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_set_expr_simple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "original".to_string(),
            field2: 10,
            field3: 20,
        }}
    )
    .await?;

    let update_id = 1i64;
    let new_value = "updated_with_expr".to_string();
    let new_field2 = 99i32;

    // Test UPDATE with SET field = value syntax
    query!(
        conn,
        UPDATE QueryTestTable SET field1 = {new_value}, field2 = {new_field2} + 5 WHERE id = {update_id}
    )
    .await?;

    // Verify update
    let results: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE id = {update_id}).await?;
    assert_eq!(results[0].field1, "updated_with_expr");
    assert_eq!(results[0].field2, 104);
    assert_eq!(results[0].field3, 20); // This shouldn't change

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_set_expr_single_field() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "original".to_string(),
            field2: 10,
            field3: 20,
        }}
    )
    .await?;

    let update_id = 1i64;
    let new_field3 = 999i64;

    // Test UPDATE with single SET field
    query!(
        conn,
        UPDATE QueryTestTable SET field3 = {new_field3} WHERE id = {update_id}
    )
    .await?;

    // Verify update
    let results: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE id = {update_id}).await?;
    assert_eq!(results[0].field1, "original");
    assert_eq!(results[0].field2, 10);
    assert_eq!(results[0].field3, 999);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_set_expr_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "original".to_string(),
            field2: 10,
            field3: 20,
        }}
    )
    .await?;

    let update_id = 1i64;
    let new_value = "returned".to_string();

    // Test UPDATE with SET field = value and RETURNING (temporarily disabled - pre-existing type errors with RETURNING)

    let returned: QueryTestData = query!(
        conn,
        UPDATE QueryTestTable SET field1 = {new_value} WHERE id = {update_id} RETURNING QueryTestData
    )
    .await?;

    assert_eq!(returned.field1, "returned");
    assert_eq!(returned.field2, 10);
    assert_eq!(returned.field3, 20);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_set_expr_literals() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<QueryTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(conn, INSERT INTO QueryTestTable VALUES {&QueryTestData {
            field1: "original".to_string(),
            field2: 10,
            field3: 20,
        }}
    )
    .await?;

    let update_id = 1i64;

    // Test UPDATE with literal values in SET
    query!(
        conn,
        UPDATE QueryTestTable SET field1 = "literal_string", field2 = 777 WHERE id = {update_id}
    )
    .await?;

    // Verify update
    let results: Vec<QueryTestData> =
        query!(conn, SELECT Vec<QueryTestData> FROM QueryTestTable WHERE id = {update_id}).await?;
    assert_eq!(results[0].field1, "literal_string");
    assert_eq!(results[0].field2, 777);

    conn.rollback().await?;
    Ok(())
}
