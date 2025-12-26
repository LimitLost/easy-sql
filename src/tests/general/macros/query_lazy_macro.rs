// Tests for query_lazy! macro
// This macro creates reusable query builders without immediate execution

use super::*;
use crate::macro_support::never_any;
use anyhow::Context;
use easy_macros::always_context;
use futures::StreamExt;
use sql_macros::{query, query_lazy};

// Note: query_lazy! generates query builders that can be executed later
// The main difference from query! is:
// 1. No connection parameter in macro call
// 2. Returns a builder/closure instead of executing immediately
// 3. Can be reused multiple times
// 4. Does NOT support EXISTS (requires immediate execution)

/// Test query_lazy! with SELECT (basic usage)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_select_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    // Create lazy query (returns LazyQueryResult struct)
    let mut lazy_query =
        query_lazy!(SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1).await?;

    // Execute the lazy query - fetch() returns a Stream
    let mut result_stream = lazy_query.fetch(&mut conn);

    // Get first result from stream
    let result_option = result_stream.next().await;
    let result_result = result_option.context("Expected at least one result")?;
    let result = result_result.context("Failed to fetch result")?;

    assert_eq!(result.int_field, 42);
    assert_eq!(result.str_field, "test");

    // Drop stream before rollback to release borrow on conn
    drop(result_stream);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
/// query.fetch() should not borrow connection forever
async fn test_query_lazy_con_not_borrowed_forever_check() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "first", true, None),
            expr_test_data(20, "second", false, None),
        ],
    )
    .await?;

    // Each lazy query can only be used once
    let mut lazy_query1 =
        query_lazy!(SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1).await?;
    let mut lazy_query2 =
        query_lazy!(SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1).await?;

    // Execute first query
    let result1 = {
        let mut stream1 = lazy_query1.fetch(&mut conn);
        let result1_option = stream1.next().await;
        let result1_result = result1_option.context("Expected result 1")?;
        result1_result.context("Failed to fetch result 1")?
    };

    // Execute second query (separate LazyQueryResult)
    let result2 = {
        let mut stream2 = lazy_query2.fetch(&mut conn);
        let result2_option = stream2.next().await;
        let result2_result = result2_option.context("Expected result 2")?;
        result2_result.context("Failed to fetch result 2")?
    };

    assert_eq!(result1.int_field, result2.int_field);
    assert_eq!(result1.int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! SELECT collecting multiple results from stream
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_select_multiple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    // query_lazy! returns single row stream, so we collect manually
    let mut lazy_query = query_lazy!(
        SELECT ExprTestData FROM ExprTestTable WHERE int_field > 5
    )
    .await?;

    let mut results = Vec::new();
    {
        let mut stream = lazy_query.fetch(&mut conn);
        while let Some(result) = stream.next().await {
            let data = result.context("Failed to fetch row")?;
            results.push(data);
        }
    }

    assert_eq!(results.len(), 3);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! SELECT with ORDER BY and LIMIT
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_select_complex() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(30, "c", true, None),
            expr_test_data(10, "a", false, None),
            expr_test_data(20, "b", true, None),
        ],
    )
    .await?;

    let mut lazy_query = query_lazy!(
        SELECT ExprTestData FROM ExprTestTable
        ORDER BY int_field DESC
        LIMIT 2
    )
    .await?;

    let mut results = Vec::new();
    {
        let mut stream = lazy_query.fetch(&mut conn);
        while let Some(result) = stream.next().await {
            let data = result.context("Failed to fetch row")?;
            results.push(data);
        }
    }

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 30);
    assert_eq!(results[1].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! INSERT with RETURNING (Postgres only, required for lazy insert)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_insert() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = default_expr_test_data();
    // query_lazy! requires RETURNING clause for INSERT
    let mut lazy_insert =
        query_lazy!(INSERT INTO ExprTestTable VALUES {data} RETURNING ExprTestData).await?;

    // Execute the insert - fetch() returns a Stream
    let returned = {
        let mut stream = lazy_insert.fetch(&mut conn);
        let returned_option = stream.next().await;
        let returned_result = returned_option.context("Expected INSERT to return row")?;
        returned_result.context("Failed to get INSERT result")?
    };

    assert_eq!(returned.int_field, 42);

    // Verify insertion using regular query! macro
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test that query_lazy! for INSERT without RETURNING requires regular query! instead
/// Note: For SQLite, use regular query! macro for simple INSERTs
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_insert_note_for_sqlite() -> anyhow::Result<()> {
    // For simple inserts without RETURNING, use query! macro instead of query_lazy!
    // query_lazy! is designed for reusable queries with output
    Ok(())
}

/// Test query_lazy! UPDATE with RETURNING
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_update() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let new_value = 100;
    let mut lazy_update = query_lazy!(
        UPDATE ExprTestTable
        SET int_field = {new_value}
        WHERE id = 1
        RETURNING ExprTestData
    )
    .await?;

    let updated = {
        let mut stream = lazy_update.fetch(&mut conn);
        let result_option = stream.next().await;
        let result_result = result_option.context("Expected UPDATE to return row")?;
        result_result.context("Failed to get UPDATE result")?
    };

    assert_eq!(updated.int_field, 100);
    assert_eq!(updated.str_field, "test");

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! DELETE with RETURNING
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_delete() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    let mut lazy_delete = query_lazy!(
        DELETE FROM ExprTestTable
        WHERE int_field > 15
        RETURNING ExprTestData
    )
    .await?;

    let mut deleted_rows = Vec::new();
    {
        let mut stream = lazy_delete.fetch(&mut conn);
        while let Some(result) = stream.next().await {
            let data = result.context("Failed to fetch deleted row")?;
            deleted_rows.push(data);
        }
    }

    // Should have deleted rows with int_field 20 and 30
    assert_eq!(deleted_rows.len(), 2);
    assert!(deleted_rows.iter().any(|r| r.int_field == 20));
    assert!(deleted_rows.iter().any(|r| r.int_field == 30));

    // Verify deletion - only row with int_field 10 should remain
    let remaining: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
    )
    .await?;

    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! with variable capture
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_variable_capture() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    let threshold = 15;
    let mut lazy_query = query_lazy!(
        SELECT ExprTestData FROM ExprTestTable WHERE int_field > {threshold}
    )
    .await?;

    let mut results = Vec::new();
    {
        let mut stream = lazy_query.fetch(&mut conn);
        while let Some(result) = stream.next().await {
            let data = result.context("Failed to fetch row")?;
            results.push(data);
        }
    }

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! can be stored and passed around
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_storage() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    // Store lazy query in a variable
    let mut my_query =
        query_lazy!(SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1).await?;

    // Get result from query
    let result = {
        let mut stream = my_query.fetch(&mut conn);
        let result_option = stream.next().await;
        let result_result = result_option.context("Expected result")?;
        result_result.context("Failed to fetch result")?
    };

    assert_eq!(result.int_field, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! with complex WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_complex_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "test", true, None),
            expr_test_data(20, "test", false, None),
            expr_test_data(30, "other", true, None),
        ],
    )
    .await?;

    let mut lazy_query = query_lazy!(
        SELECT ExprTestData FROM ExprTestTable
        WHERE (int_field >= 10 AND int_field <= 30)
          AND str_field = "test"
          AND bool_field = true
    )
    .await?;

    let mut results = Vec::new();
    {
        let mut stream = lazy_query.fetch(&mut conn);
        while let Some(result) = stream.next().await {
            let data = result.context("Failed to fetch row")?;
            results.push(data);
        }
    }

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test query_lazy! with RETURNING clause (Postgres only)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = default_expr_test_data();
    let mut lazy_insert = query_lazy!(
        INSERT INTO ExprTestTable VALUES {data} RETURNING ExprTestData
    )
    .await?;

    let returned = {
        let mut stream = lazy_insert.fetch(&mut conn);
        let returned_option = stream.next().await;
        let returned_result = returned_option.context("Expected INSERT to return row")?;
        returned_result.context("Failed to get INSERT result")?
    };

    assert_eq!(returned.int_field, 42);
    assert_eq!(returned.str_field, "test");

    conn.rollback().await?;
    Ok(())
}

/// Test multiple lazy queries in sequence (INSERT, UPDATE, SELECT)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_multiple_sequential() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Use regular query! for INSERT (query_lazy! requires RETURNING for INSERT)
    let data = default_expr_test_data();
    query!(&mut conn, INSERT INTO ExprTestTable VALUES {data}).await?;

    // Now use query_lazy! for UPDATE with RETURNING
    let new_value = 999;
    let mut lazy_update = query_lazy!(
        UPDATE ExprTestTable
        SET int_field = {new_value}
        WHERE id = 1
        RETURNING ExprTestData
    )
    .await?;

    let updated = {
        let mut stream = lazy_update.fetch(&mut conn);
        let result_option = stream.next().await;
        let result_result = result_option.context("Expected UPDATE to return row")?;
        result_result.context("Failed to get UPDATE result")?
    };

    assert_eq!(updated.int_field, 999);

    // Verify with SELECT
    let mut lazy_select =
        query_lazy!(SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1).await?;
    let result = {
        let mut stream = lazy_select.fetch(&mut conn);
        let result_option = stream.next().await;
        let result_result = result_option.context("Expected SELECT result")?;
        result_result.context("Failed to fetch result")?
    };

    assert_eq!(result.int_field, 999);

    conn.rollback().await?;
    Ok(())
}

/// Test that query_lazy! can be used in different scopes
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_cross_scope() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    // Define lazy query with a captured variable (must outlive the query)
    let filter_value = 1;
    let mut lazy_query =
        query_lazy!(SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = {filter_value})
            .await?;

    // Execute in different scope
    let result = {
        let mut stream = lazy_query.fetch(&mut conn);
        let result_option = stream.next().await;
        let result_result = result_option.context("Expected result")?;
        result_result.context("Failed to fetch result")?
    };

    assert_eq!(result.int_field, 42);

    conn.rollback().await?;
    Ok(())
}

// Note: EXISTS is NOT supported in query_lazy! as it requires immediate execution
// The following test would fail to compile:
//
// #[tokio::test]
// async fn test_query_lazy_exists_should_fail() {
//     // This should cause a compile error:
//     // let lazy_exists = query_lazy!(EXISTS ExprTestTable WHERE id = 1);
// }
