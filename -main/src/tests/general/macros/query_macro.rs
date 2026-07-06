// Comprehensive tests for query! macro
// Tests all query types: SELECT, INSERT, UPDATE, DELETE, EXISTS

use super::*;
use anyhow::Context;
use easy_macros::{always_context /* always_context_debug as always_context */};
use easy_sql_macros::query;
use serde::{Deserialize, Serialize};

fn assert_update_set_error<T>(result: anyhow::Result<T>, expected_substring: &str) {
    assert!(result.is_err());
    let error_message = format!("{:#}", result.err().unwrap());
    assert!(
        error_message.contains(expected_substring),
        "Unexpected error: {error_message}"
    );
}

// ==============================================
// 1. SELECT QUERIES
// ==============================================

#[always_context(skip(!))]
/// Test simple SELECT returning single row
// #[always_context_debug(skip(!))]
#[tokio::test]
async fn test_query_select_single_row() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;

    assert_eq!(result.int_field, 42);
    assert_eq!(result.str_field, "test");

    conn.rollback().await?;
    Ok(())
}

/// Test query! using sqlx::Pool as connection
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_with_sqlx_pool_connection() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ExprTestTable>().await?;
    let mut pool = pool_resource.pool();

    let data = default_expr_test_data();
    query!(pool, INSERT INTO ExprTestTable VALUES {data}).await?;

    let result: ExprTestData = query!(pool,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;

    assert_eq!(result.int_field, 42);
    assert_eq!(result.str_field, "test");

    Ok(())
}

/// Test SELECT returning Vec<T>
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_multiple_rows() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field > 5
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with ORDER BY single column
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_order_by() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true ORDER BY int_field
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with ORDER BY DESC
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_order_by_desc() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true ORDER BY int_field DESC
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 30);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with LIMIT
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true LIMIT 2
    )
    .await?;

    assert_eq!(results.len(), 2);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with ORDER BY and LIMIT combined
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_order_by_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field > 5
        ORDER BY int_field DESC LIMIT 2
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 40);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with LIMIT and OFFSET combined
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_limit_offset() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let limit_rows = 2;
    let offset_rows = 1;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE true
        ORDER BY int_field ASC
        LIMIT {limit_rows}
        OFFSET {offset_rows}
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT accepts OFFSET before LIMIT in macro input
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_offset_before_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let limit_rows = 2;
    let offset_rows = 1;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE true
        ORDER BY int_field ASC
        OFFSET {offset_rows}
        LIMIT {limit_rows}
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT DISTINCT
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_distinct() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "same", true, None),
            expr_test_data(10, "same", true, None),
            expr_test_data(20, "different", false, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT DISTINCT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    // DISTINCT should reduce duplicates
    assert!(results.len() <= 3);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with complex WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_complex_where() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (int_field >= 10 AND int_field <= 30)
          AND str_field = "test"
          AND bool_field = true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT returning empty result (single row expected - should error)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_no_row_found_error() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let result: Result<ExprTestData, _> = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 99999
    )
    .await;

    assert!(
        result.is_err(),
        "Should error when no row found for single row query"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT returning empty Vec (should succeed with empty vec)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_empty_vec() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE ExprTestTable.id = 99999
    )
    .await?;

    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 2. INSERT QUERIES
// ==============================================

/// Test simple INSERT
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert_single() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = default_expr_test_data();
    query!(&mut conn, INSERT INTO ExprTestTable VALUES {data}).await?;

    // Verify insert succeeded
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test INSERT with RETURNING clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = default_expr_test_data();
    let returned: ExprTestData = query!(&mut conn,
        INSERT INTO ExprTestTable VALUES {data} RETURNING ExprTestData
    )
    .await?;

    assert_eq!(returned.int_field, 42);
    assert_eq!(returned.str_field, "test");

    conn.rollback().await?;
    Ok(())
}

/// Test bulk INSERT
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert_multiple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data_vec = vec![
        expr_test_data(10, "a", true, None),
        expr_test_data(20, "b", false, None),
        expr_test_data(30, "c", true, None),
    ];

    for data in data_vec {
        query!(&mut conn, INSERT INTO ExprTestTable VALUES {data}).await?;
    }

    // Verify all inserted
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 3);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3. UPDATE QUERIES
// ==============================================

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct MaybeUpdateTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    optional_text: Option<String>,
    optional_number: Option<i32>,
}

#[derive(Insert)]
#[sql(table = MaybeUpdateTable)]
#[sql(default = id)]
struct MaybeUpdateInsert {
    name: String,
    optional_text: Option<String>,
    optional_number: Option<i32>,
}

/// Test inserting Option<T> columns with T values
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert_option_table_from_value() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    #[derive(Insert)]
    #[sql(table = MaybeUpdateTable)]
    #[sql(default = id)]
    struct MaybeUpdateInsertValue {
        name: String,
        optional_text: String,
        optional_number: i32,
    }

    let data = MaybeUpdateInsertValue {
        name: "inserted".to_string(),
        optional_text: "value".to_string(),
        optional_number: 42,
    };

    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.name, "inserted");
    assert_eq!(row.optional_text, Some("value".to_string()));
    assert_eq!(row.optional_number, Some(42));

    conn.rollback().await?;
    Ok(())
}

/// Test simple UPDATE
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_single() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert initial data
    insert_test_data(&mut conn, default_expr_test_data()).await?;

    // Update data
    let updated_data = expr_test_data(99, "updated", false, Some("new"));
    query!(&mut conn, UPDATE ExprTestTable SET {updated_data} WHERE id = 1).await?;

    // Verify update
    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;

    assert_eq!(result.int_field, 99);
    assert_eq!(result.str_field, "updated");
    assert!(!result.bool_field);
    assert_eq!(result.nullable_field, Some("new".to_string()));

    conn.rollback().await?;
    Ok(())
}

#[derive(Update)]
#[sql(table = ExprTestTable)]
struct CollectionPayloadUpdate {
    int_field: i32,
    str_field: String,
}

#[derive(Update)]
#[sql(table = ExprTestTable)]
struct CollectionNoAssignmentsUpdate {
    #[sql(maybe_update)]
    nullable_field: Option<Option<String>>,
}

macro_rules! run_update_collection_success_case {
    ($payload_expr:expr, $expected_int:expr, $expected_str:expr) => {{
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        insert_test_data(&mut conn, default_expr_test_data()).await?;

        query!(&mut conn,
            UPDATE ExprTestTable SET {$payload_expr} WHERE ExprTestTable.id = 1
        )
        .await?;

        let result: ExprTestData = query!(&mut conn,
            SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
        )
        .await?;

        assert_eq!(result.int_field, $expected_int);
        assert_eq!(result.str_field, $expected_str);

        conn.rollback().await?;
        Ok(())
    }};
}

macro_rules! run_update_collection_error_case {
    ($payload_expr:expr, $expected_error:expr) => {{
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        insert_test_data(&mut conn, default_expr_test_data()).await?;

        let result = query!(&mut conn,
            UPDATE ExprTestTable SET {$payload_expr} WHERE ExprTestTable.id = 1
        )
        .await;

        assert_update_set_error(result, $expected_error);

        conn.rollback().await?;
        Ok(())
    }};
}

/// Test UPDATE with vector payload support
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vector_payload() -> anyhow::Result<()> {
    let updates = vec![CollectionPayloadUpdate {
        int_field: 123,
        str_field: "vector-updated".to_string(),
    }];
    run_update_collection_success_case!(updates, 123, "vector-updated")
}

/// Test UPDATE with &Vec<T> payload support
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vec_ref_payload() -> anyhow::Result<()> {
    let updates = vec![CollectionPayloadUpdate {
        int_field: 124,
        str_field: "ref-vec-updated".to_string(),
    }];
    run_update_collection_success_case!(&updates, 124, "ref-vec-updated")
}

/// Test UPDATE with &[T] payload support
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_slice_payload() -> anyhow::Result<()> {
    let updates = [CollectionPayloadUpdate {
        int_field: 125,
        str_field: "slice-updated".to_string(),
    }];
    run_update_collection_success_case!(&updates[..], 125, "slice-updated")
}

/// Test UPDATE with multi-item vector merge behavior
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vector_payload_multi_item_merge() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: Some(10),
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MergeVectorUpdate {
        #[sql(maybe_update)]
        optional_text: Option<String>,
        #[sql(maybe_update)]
        optional_number: Option<i32>,
    }

    let updates = vec![
        MergeVectorUpdate {
            optional_text: Some("merged-text".to_string()),
            optional_number: None,
        },
        MergeVectorUpdate {
            optional_text: None,
            optional_number: Some(77),
        },
    ];

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {updates} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let result: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(result.optional_text, Some("merged-text".to_string()));
    assert_eq!(result.optional_number, Some(77));

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE rejects empty vector payload
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vector_payload_empty_rejected() -> anyhow::Result<()> {
    let updates: Vec<CollectionNoAssignmentsUpdate> = Vec::new();
    run_update_collection_error_case!(updates, "UPDATE ... SET {collection} cannot be empty")
}

/// Test UPDATE rejects vectors that produce no assignments
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vector_payload_no_assignments_rejected() -> anyhow::Result<()> {
    let updates = vec![CollectionNoAssignmentsUpdate {
        nullable_field: None,
    }];
    run_update_collection_error_case!(
        updates,
        "UPDATE ... SET {collection} produced no assignments"
    )
}

/// Test UPDATE rejects single-item payloads that produce no assignments
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_single_payload_no_assignments_rejected() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    #[derive(Update)]
    #[sql(table = ExprTestTable)]
    struct EmptyAssignmentsSingleUpdate {
        #[sql(maybe_update)]
        nullable_field: Option<Option<String>>,
    }

    let update = EmptyAssignmentsSingleUpdate {
        nullable_field: None,
    };

    let result = query!(&mut conn,
        UPDATE ExprTestTable SET {update} WHERE ExprTestTable.id = 1
    )
    .await;

    assert_update_set_error(result, "UPDATE ... SET {data} produced no assignments");

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE rejects empty &Vec<T> payload
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vec_ref_payload_empty_rejected() -> anyhow::Result<()> {
    let updates: Vec<CollectionNoAssignmentsUpdate> = Vec::new();
    run_update_collection_error_case!(&updates, "UPDATE ... SET {collection} cannot be empty")
}

/// Test UPDATE rejects vectors by reference that produce no assignments
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_vec_ref_payload_no_assignments_rejected() -> anyhow::Result<()> {
    let updates = vec![CollectionNoAssignmentsUpdate {
        nullable_field: None,
    }];
    run_update_collection_error_case!(
        &updates,
        "UPDATE ... SET {collection} produced no assignments"
    )
}

/// Test UPDATE rejects empty &[T] payload
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_slice_payload_empty_rejected() -> anyhow::Result<()> {
    let updates: [CollectionNoAssignmentsUpdate; 0] = [];
    run_update_collection_error_case!(&updates[..], "UPDATE ... SET {collection} cannot be empty")
}

/// Test UPDATE rejects slices that produce no assignments
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_slice_payload_no_assignments_rejected() -> anyhow::Result<()> {
    let updates = [CollectionNoAssignmentsUpdate {
        nullable_field: None,
    }];
    run_update_collection_error_case!(
        &updates[..],
        "UPDATE ... SET {collection} produced no assignments"
    )
}

#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_lock_modes_postgres() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let row_for_update: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1 FOR UPDATE
    )
    .await?;
    assert_eq!(row_for_update.int_field, 42);

    let row_for_no_key_update: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1 FOR NO KEY UPDATE
    )
    .await?;
    assert_eq!(row_for_no_key_update.int_field, 42);

    let row_for_share: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1 FOR SHARE
    )
    .await?;
    assert_eq!(row_for_share.int_field, 42);

    let row_for_key_share: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1 FOR KEY SHARE
    )
    .await?;
    assert_eq!(row_for_key_share.int_field, 42);

    conn.rollback().await?;
    Ok(())
}

/// `FOR UPDATE` actually takes a row lock: while one transaction holds it, a concurrent writer targeting the
/// same row must block until the lock is released, then apply its write.
#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_for_update_blocks_concurrent_writer() -> anyhow::Result<()> {
    let db = std::sync::Arc::new(Database::setup_for_testing::<ExprTestTable>().await?);

    // Seed one row (id = 1, int_field = 42).
    {
        let mut conn = db.conn().await?;
        insert_test_data(&mut conn, default_expr_test_data()).await?;
    }

    // Transaction A: lock row 1 with FOR UPDATE and keep the transaction open (lock held).
    let mut tx_a = db.transaction().await?;
    let _locked: ExprTestData = query!(&mut tx_a,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1 FOR UPDATE
    )
    .await?;

    // Transaction B on its own connection: UPDATE row 1. Postgres must block it on A's row lock.
    let db_b = db.clone();
    let writer = tokio::spawn(async move {
        let mut tx_b = db_b.transaction().await?;
        query!(&mut tx_b,
            UPDATE ExprTestTable SET int_field = 99 WHERE ExprTestTable.id = 1
        )
        .await?;
        tx_b.commit().await?;
        anyhow::Ok(())
    });

    // Give B time to reach the lock. It cannot finish while A holds it, so it must still be pending.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    assert!(
        !writer.is_finished(),
        "FOR UPDATE must block the concurrent writer while the lock is held"
    );

    // Release the lock; B unblocks, applies its update, and commits.
    tx_a.rollback().await?;
    let writer_result = writer.await.context("blocked writer task panicked")?;
    writer_result.context("blocked writer failed to update after unblocking")?;

    // The write landed only after the lock was released.
    let mut conn = db.conn().await?;
    let row: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;
    assert_eq!(row.int_field, 99, "the blocked writer applied its update once unblocked");

    Ok(())
}

/// Test updating Option<T> columns with T values
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_option_table_from_value() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: Some(10),
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct UpdateOptionalFromValue {
        name: String,
        optional_text: String,
    }

    let update = UpdateOptionalFromValue {
        name: "updated".to_string(),
        optional_text: "changed".to_string(),
    };
    let update_ref = &update;

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update_ref} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.name, "updated");
    assert_eq!(row.optional_text, Some("changed".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update skips Option<T> None
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_maybe_update_option_skip_none() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: Some(10),
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MaybeUpdateOption {
        name: String,
        #[sql(maybe_update)]
        optional_text: Option<String>,
    }

    let update = MaybeUpdateOption {
        name: "updated".to_string(),
        optional_text: None,
    };

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.name, "updated");
    assert_eq!(row.optional_text, Some("keep".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update updates Option<T> Some
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_maybe_update_option_some() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: None,
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MaybeUpdateOption {
        name: String,
        #[sql(maybe_update)]
        optional_text: Option<String>,
    }

    let update = MaybeUpdateOption {
        name: "updated".to_string(),
        optional_text: Some("changed".to_string()),
    };

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.optional_text, Some("changed".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update Option<Option<T>> sets NULL
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_maybe_update_option_option_set_null() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: Some(10),
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MaybeUpdateOptionOption {
        name: String,
        #[sql(maybe_update)]
        optional_text: Option<Option<String>>,
    }

    let update = MaybeUpdateOptionOption {
        name: "updated".to_string(),
        optional_text: Some(None),
    };

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.optional_text, None);

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update  Option<T> Some(Some) updates on Nullable value
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_maybe_update_nullable_some_without_nesting() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: Some(10),
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MaybeUpdateOptionOption {
        name: String,
        #[sql(maybe_update)]
        optional_text: Option<String>,
    }

    let update = MaybeUpdateOptionOption {
        name: "updated".to_string(),
        optional_text: Some("changed".to_string()),
    };

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.optional_text, Some("changed".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update Option<Option<T>> None skips update
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_maybe_update_option_option_skip() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: None,
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MaybeUpdateOptionOption {
        name: String,
        #[sql(maybe_update)]
        optional_text: Option<Option<String>>,
    }

    let update = MaybeUpdateOptionOption {
        name: "updated".to_string(),
        optional_text: None,
    };

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.optional_text, Some("keep".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update across multiple fields
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_maybe_update_multiple_fields() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MaybeUpdateTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateInsert {
        name: "original".to_string(),
        optional_text: Some("keep".to_string()),
        optional_number: Some(9),
    };
    query!(&mut conn, INSERT INTO MaybeUpdateTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTable)]
    struct MaybeUpdateMulti {
        name: String,
        #[sql(maybe_update)]
        optional_text: Option<String>,
        #[sql(maybe_update)]
        optional_number: Option<i32>,
    }

    let update = MaybeUpdateMulti {
        name: "multi".to_string(),
        optional_text: Some("changed".to_string()),
        optional_number: None,
    };
    let update_ref = &update;

    query!(&mut conn,
        UPDATE MaybeUpdateTable SET {update_ref} WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTable = query!(&mut conn,
        SELECT MaybeUpdateTable FROM MaybeUpdateTable WHERE MaybeUpdateTable.id = 1
    )
    .await?;

    assert_eq!(row.name, "multi");
    assert_eq!(row.optional_text, Some("changed".to_string()));
    assert_eq!(row.optional_number, Some(9));

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with WHERE clause matching multiple rows
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_multiple_rows() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "old", true, None),
            expr_test_data(20, "old", false, None),
            expr_test_data(30, "other", true, None),
        ],
    )
    .await?;

    // Update all rows with str_field = "old"
    let updated_data = expr_test_data(100, "new", true, None);
    query!(&mut conn, UPDATE ExprTestTable SET {updated_data} WHERE str_field = "old").await?;

    // Verify updates
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE str_field = "new"
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.int_field == 100));

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.1 BYTES QUERIES
// ==============================================

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct BytesPayload {
    label: String,
    data: Vec<u8>,
}

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct BytesTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(bytes)]
    payload: BytesPayload,
    #[sql(bytes)]
    optional_payload: Option<BytesPayload>,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = BytesTestTable)]
#[sql(default = id)]
struct BytesTestData {
    #[sql(bytes)]
    payload: BytesPayload,
    #[sql(bytes)]
    optional_payload: Option<BytesPayload>,
}

/// Test bytes roundtrip with payloads
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_roundtrip_payload() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let payload = BytesPayload {
        label: "payload".to_string(),
        data: vec![10, 11, 12, 13],
    };
    let data = BytesTestData {
        payload: payload.clone(),
        optional_payload: Some(BytesPayload {
            label: "optional".to_string(),
            data: vec![9, 8, 7],
        }),
    };

    query!(&mut conn, INSERT INTO BytesTestTable VALUES {data}).await?;

    let row: BytesTestData = query!(&mut conn,
        SELECT BytesTestData FROM BytesTestTable WHERE BytesTestTable.id = 1
    )
    .await?;

    assert_eq!(row.payload, payload);
    assert_eq!(
        row.optional_payload,
        Some(BytesPayload {
            label: "optional".to_string(),
            data: vec![9, 8, 7],
        })
    );

    conn.rollback().await?;
    Ok(())
}

/// Test bytes roundtrip with None and empty blobs
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_roundtrip_none_and_empty() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = BytesTestData {
        payload: BytesPayload {
            label: "empty".to_string(),
            data: Vec::new(),
        },
        optional_payload: None,
    };

    query!(&mut conn, INSERT INTO BytesTestTable VALUES {data}).await?;

    let row: BytesTestData = query!(&mut conn,
        SELECT BytesTestData FROM BytesTestTable WHERE BytesTestTable.id = 1
    )
    .await?;

    assert_eq!(row.payload.data, Vec::<u8>::new());
    assert_eq!(row.optional_payload, None);

    conn.rollback().await?;
    Ok(())
}

/// Test bytes update replacing payloads and raw bytes
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_update_payload() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = BytesTestData {
        payload: BytesPayload {
            label: "start".to_string(),
            data: vec![1],
        },
        optional_payload: Some(BytesPayload {
            label: "optional".to_string(),
            data: vec![2],
        }),
    };
    query!(&mut conn, INSERT INTO BytesTestTable VALUES {data}).await?;

    let updated = BytesTestData {
        payload: BytesPayload {
            label: "updated".to_string(),
            data: vec![10, 20, 30],
        },
        optional_payload: None,
    };

    query!(&mut conn,
        UPDATE BytesTestTable SET {updated} WHERE BytesTestTable.id = 1
    )
    .await?;

    let row: BytesTestData = query!(&mut conn,
        SELECT BytesTestData FROM BytesTestTable WHERE BytesTestTable.id = 1
    )
    .await?;

    assert_eq!(row.payload.data, vec![10, 20, 30]);
    assert_eq!(row.optional_payload, None);

    conn.rollback().await?;
    Ok(())
}

/// Test bytes update with large payloads
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_update_large_payload() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = BytesTestData {
        payload: BytesPayload {
            label: "large".to_string(),
            data: vec![1, 2, 3],
        },
        optional_payload: None,
    };
    query!(&mut conn, INSERT INTO BytesTestTable VALUES {data}).await?;

    let large_blob = vec![42u8; 16 * 1024];
    let updated = BytesTestData {
        payload: BytesPayload {
            label: "large".to_string(),
            data: large_blob.clone(),
        },
        optional_payload: Some(BytesPayload {
            label: "nested".to_string(),
            data: vec![5, 6, 7],
        }),
    };

    query!(&mut conn,
        UPDATE BytesTestTable SET {updated} WHERE BytesTestTable.id = 1
    )
    .await?;

    let row: BytesTestData = query!(&mut conn,
        SELECT BytesTestData FROM BytesTestTable WHERE BytesTestTable.id = 1
    )
    .await?;

    assert_eq!(row.payload.data, large_blob);
    assert_eq!(row.optional_payload.as_ref().unwrap().data.len(), 3);

    conn.rollback().await?;
    Ok(())
}

/// Test bytes update with maybe_update on optional payload
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_maybe_update_optional_payload() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = BytesTestData {
        payload: BytesPayload {
            label: "start".to_string(),
            data: vec![1, 2, 3],
        },
        optional_payload: Some(BytesPayload {
            label: "keep".to_string(),
            data: vec![4, 5],
        }),
    };
    query!(&mut conn, INSERT INTO BytesTestTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = BytesTestTable)]
    struct BytesMaybeUpdate {
        #[sql(bytes)]
        payload: BytesPayload,
        #[sql(bytes)]
        #[sql(maybe_update)]
        optional_payload: Option<Option<BytesPayload>>,
    }

    let updated = BytesMaybeUpdate {
        payload: BytesPayload {
            label: "updated".to_string(),
            data: vec![9, 9],
        },
        optional_payload: Some(None),
    };

    query!(&mut conn,
        UPDATE BytesTestTable SET {updated} WHERE BytesTestTable.id = 1
    )
    .await?;

    let row: BytesTestData = query!(&mut conn,
        SELECT BytesTestData FROM BytesTestTable WHERE BytesTestTable.id = 1
    )
    .await?;

    assert_eq!(row.payload.label, "updated");
    assert_eq!(row.optional_payload, None);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.2 ADVANCED BYTES TESTS - Basic Serde Types
// ==============================================

/// Test bytes with HashMap<String, String>
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_hashmap_string_string() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut map = std::collections::HashMap::new();
    map.insert("key1".to_string(), "value1".to_string());
    map.insert("key2".to_string(), "value2".to_string());
    map.insert("key3".to_string(), "value3".to_string());

    let data = BytesTestData {
        payload: BytesPayload {
            label: "hashmap_test".to_string(),
            data: vec![],
        },
        optional_payload: Some(BytesPayload {
            label: format!("{:?}", map),
            data: vec![],
        }),
    };

    query!(&mut conn, INSERT INTO BytesTestTable VALUES {data}).await?;

    let row: BytesTestData = query!(&mut conn,
        SELECT BytesTestData FROM BytesTestTable WHERE BytesTestTable.id = 1
    )
    .await?;

    // Verify the payload was stored and retrieved correctly
    assert_eq!(row.payload.label, "hashmap_test");
    assert!(row.optional_payload.is_some());
    assert!(row.optional_payload.unwrap().label.contains("key1"));

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with BTreeMap for ordered key storage
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_btreemap_ordered() -> anyhow::Result<()> {
    use std::collections::BTreeMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct BTreePayload {
        ordered_data: BTreeMap<String, i64>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct BTreeTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        payload: BTreePayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = BTreeTestTable)]
    #[sql(default = id)]
    struct BTreeTestData {
        #[sql(bytes)]
        payload: BTreePayload,
    }

    let db = Database::setup_for_testing::<BTreeTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut map = BTreeMap::new();
    map.insert("zebra".to_string(), 3);
    map.insert("apple".to_string(), 1);
    map.insert("mango".to_string(), 2);

    let data = BTreeTestData {
        payload: BTreePayload {
            ordered_data: map.clone(),
        },
    };

    query!(&mut conn, INSERT INTO BTreeTestTable VALUES {data}).await?;

    let row: BTreeTestData = query!(&mut conn,
        SELECT BTreeTestData FROM BTreeTestTable WHERE BTreeTestTable.id = 1
    )
    .await?;

    // BTreeMap maintains sorted order
    let keys: Vec<_> = row.payload.ordered_data.keys().collect();
    assert_eq!(keys, vec!["apple", "mango", "zebra"]);
    assert_eq!(row.payload.ordered_data.get("apple"), Some(&1));

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with Vec of custom structs
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_vec_custom_struct() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct PathPayload {
        points: Vec<Point>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct PathTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        path: PathPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = PathTestTable)]
    #[sql(default = id)]
    struct PathTestData {
        #[sql(bytes)]
        path: PathPayload,
    }

    let db = Database::setup_for_testing::<PathTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = PathTestData {
        path: PathPayload {
            points: vec![
                Point { x: 0, y: 0 },
                Point { x: 10, y: 20 },
                Point { x: 30, y: 40 },
                Point { x: 50, y: 60 },
            ],
        },
    };

    query!(&mut conn, INSERT INTO PathTestTable VALUES {data}).await?;

    let row: PathTestData = query!(&mut conn,
        SELECT PathTestData FROM PathTestTable WHERE PathTestTable.id = 1
    )
    .await?;

    assert_eq!(row.path.points.len(), 4);
    assert_eq!(row.path.points[0], Point { x: 0, y: 0 });
    assert_eq!(row.path.points[3], Point { x: 50, y: 60 });

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with nested collections: Vec<HashMap<String, i32>>
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_nested_vec_hashmap() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct NestedPayload {
        maps: Vec<HashMap<String, i32>>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct NestedTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: NestedPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = NestedTestTable)]
    #[sql(default = id)]
    struct NestedTestData {
        #[sql(bytes)]
        data: NestedPayload,
    }

    let db = Database::setup_for_testing::<NestedTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut map1 = HashMap::new();
    map1.insert("a".to_string(), 1);
    map1.insert("b".to_string(), 2);

    let mut map2 = HashMap::new();
    map2.insert("c".to_string(), 3);
    map2.insert("d".to_string(), 4);

    let data = NestedTestData {
        data: NestedPayload {
            maps: vec![map1, map2],
        },
    };

    query!(&mut conn, INSERT INTO NestedTestTable VALUES {data}).await?;

    let row: NestedTestData = query!(&mut conn,
        SELECT NestedTestData FROM NestedTestTable WHERE NestedTestTable.id = 1
    )
    .await?;

    assert_eq!(row.data.maps.len(), 2);
    assert_eq!(row.data.maps[0].get("a"), Some(&1));
    assert_eq!(row.data.maps[1].get("d"), Some(&4));

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with HashMap containing Vec<u8> values
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_hashmap_vec_u8_values() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct BlobMapPayload {
        blobs: HashMap<String, Vec<u8>>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct BlobMapTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: BlobMapPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = BlobMapTestTable)]
    #[sql(default = id)]
    struct BlobMapTestData {
        #[sql(bytes)]
        data: BlobMapPayload,
    }

    let db = Database::setup_for_testing::<BlobMapTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut blobs = HashMap::new();
    blobs.insert("image".to_string(), vec![0xFF, 0xD8, 0xFF, 0xE0]);
    blobs.insert("text".to_string(), b"hello".to_vec());
    blobs.insert("empty".to_string(), vec![]);

    let data = BlobMapTestData {
        data: BlobMapPayload {
            blobs: blobs.clone(),
        },
    };

    query!(&mut conn, INSERT INTO BlobMapTestTable VALUES {data}).await?;

    let row: BlobMapTestData = query!(&mut conn,
        SELECT BlobMapTestData FROM BlobMapTestTable WHERE BlobMapTestTable.id = 1
    )
    .await?;

    assert_eq!(row.data.blobs.get("image"), Some(&vec![0xFF, 0xD8, 0xFF, 0xE0]));
    assert_eq!(row.data.blobs.get("text"), Some(&b"hello".to_vec()));
    assert_eq!(row.data.blobs.get("empty"), Some(&vec![]));

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.3 ADVANCED BYTES TESTS - Edge Cases
// ==============================================

/// Test bytes with deeply nested structures (4 levels)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_deeply_nested() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct Level4 {
        value: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct Level3 {
        items: Vec<Level4>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct Level2 {
        map: HashMap<String, Level3>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct DeepPayload {
        level1: Level2,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct DeepTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: DeepPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = DeepTestTable)]
    #[sql(default = id)]
    struct DeepTestData {
        #[sql(bytes)]
        data: DeepPayload,
    }

    let db = Database::setup_for_testing::<DeepTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut inner_map = HashMap::new();
    inner_map.insert(
        "key1".to_string(),
        Level3 {
            items: vec![
                Level4 {
                    value: "deep1".to_string(),
                },
                Level4 {
                    value: "deep2".to_string(),
                },
            ],
        },
    );

    let data = DeepTestData {
        data: DeepPayload {
            level1: Level2 {
                map: inner_map,
            },
        },
    };

    query!(&mut conn, INSERT INTO DeepTestTable VALUES {data}).await?;

    let row: DeepTestData = query!(&mut conn,
        SELECT DeepTestData FROM DeepTestTable WHERE DeepTestTable.id = 1
    )
    .await?;

    let retrieved_value = &row.data.level1.map.get("key1").unwrap().items[0].value;
    assert_eq!(retrieved_value, "deep1");

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with very large payload (1MB)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_very_large_payload() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct LargePayload {
        data: Vec<u8>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct LargeTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        blob: LargePayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = LargeTestTable)]
    #[sql(default = id)]
    struct LargeTestData {
        #[sql(bytes)]
        blob: LargePayload,
    }

    let db = Database::setup_for_testing::<LargeTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Create 1MB of data
    let large_data = vec![0xABu8; 1024 * 1024];

    let data = LargeTestData {
        blob: LargePayload {
            data: large_data.clone(),
        },
    };

    query!(&mut conn, INSERT INTO LargeTestTable VALUES {data}).await?;

    let row: LargeTestData = query!(&mut conn,
        SELECT LargeTestData FROM LargeTestTable WHERE LargeTestTable.id = 1
    )
    .await?;

    assert_eq!(row.blob.data.len(), 1024 * 1024);
    assert_eq!(row.blob.data[0], 0xAB);
    assert_eq!(row.blob.data[1024 * 1024 - 1], 0xAB);

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with unicode content including emojis
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_unicode_emojis() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct UnicodePayload {
        labels: HashMap<String, String>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct UnicodeTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: UnicodePayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = UnicodeTestTable)]
    #[sql(default = id)]
    struct UnicodeTestData {
        #[sql(bytes)]
        data: UnicodePayload,
    }

    let db = Database::setup_for_testing::<UnicodeTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut labels = HashMap::new();
    labels.insert("emoji_1".to_string(), "🚀🎉🔥".to_string());
    labels.insert("chinese".to_string(), "你好世界".to_string());
    labels.insert("arabic".to_string(), "مرحبا بالعالم".to_string());
    labels.insert("russian".to_string(), "Привет мир".to_string());
    labels.insert("mixed".to_string(), "Hello🌍世界".to_string());

    let data = UnicodeTestData {
        data: UnicodePayload {
            labels: labels.clone(),
        },
    };

    query!(&mut conn, INSERT INTO UnicodeTestTable VALUES {data}).await?;

    let row: UnicodeTestData = query!(&mut conn,
        SELECT UnicodeTestData FROM UnicodeTestTable WHERE UnicodeTestTable.id = 1
    )
    .await?;

    assert_eq!(row.data.labels.get("emoji_1"), Some(&"🚀🎉🔥".to_string()));
    assert_eq!(row.data.labels.get("chinese"), Some(&"你好世界".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with numeric extremes
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_numeric_extremes() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct NumericPayload {
        i64_min: i64,
        i64_max: i64,
        u64_max: u64,
        f64_zero: f64,
    }

    // Distinct struct name: the shared test suite already defines a `NumericTestTable` (macros/mod.rs), and table
    // names (derived from the struct name) are globally unique across the crate, so this local table is renamed.
    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct NumericExtremesTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: NumericPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = NumericExtremesTable)]
    #[sql(default = id)]
    struct NumericTestData {
        #[sql(bytes)]
        data: NumericPayload,
    }

    let db = Database::setup_for_testing::<NumericExtremesTable>().await?;
    let mut conn = db.transaction().await?;

    let data = NumericTestData {
        data: NumericPayload {
            i64_min: i64::MIN,
            i64_max: i64::MAX,
            u64_max: u64::MAX,
            f64_zero: 0.0,
        },
    };

    query!(&mut conn, INSERT INTO NumericExtremesTable VALUES {data}).await?;

    let row: NumericTestData = query!(&mut conn,
        SELECT NumericTestData FROM NumericExtremesTable WHERE NumericExtremesTable.id = 1
    )
    .await?;

    assert_eq!(row.data.i64_min, i64::MIN);
    assert_eq!(row.data.i64_max, i64::MAX);
    assert_eq!(row.data.u64_max, u64::MAX);
    assert_eq!(row.data.f64_zero, 0.0);

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with empty collections
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_empty_collections() -> anyhow::Result<()> {
    use std::collections::{HashMap, BTreeMap};

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct EmptyCollectionsPayload {
        empty_vec: Vec<String>,
        empty_hashmap: HashMap<String, i32>,
        empty_btreemap: BTreeMap<String, i32>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct EmptyCollectionsTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: EmptyCollectionsPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = EmptyCollectionsTestTable)]
    #[sql(default = id)]
    struct EmptyCollectionsTestData {
        #[sql(bytes)]
        data: EmptyCollectionsPayload,
    }

    let db = Database::setup_for_testing::<EmptyCollectionsTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = EmptyCollectionsTestData {
        data: EmptyCollectionsPayload {
            empty_vec: vec![],
            empty_hashmap: HashMap::new(),
            empty_btreemap: BTreeMap::new(),
        },
    };

    query!(&mut conn, INSERT INTO EmptyCollectionsTestTable VALUES {data}).await?;

    let row: EmptyCollectionsTestData = query!(&mut conn,
        SELECT EmptyCollectionsTestData FROM EmptyCollectionsTestTable WHERE EmptyCollectionsTestTable.id = 1
    )
    .await?;

    assert!(row.data.empty_vec.is_empty());
    assert!(row.data.empty_hashmap.is_empty());
    assert!(row.data.empty_btreemap.is_empty());

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.4 ADVANCED BYTES TESTS - Option Combinations
// ==============================================

/// Test bytes with Option<HashMap>
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_option_hashmap() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct OptionMapPayload {
        data: HashMap<String, String>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct OptionMapTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        required: OptionMapPayload,
        #[sql(bytes)]
        optional: Option<OptionMapPayload>,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = OptionMapTestTable)]
    #[sql(default = id)]
    struct OptionMapTestData {
        #[sql(bytes)]
        required: OptionMapPayload,
        #[sql(bytes)]
        optional: Option<OptionMapPayload>,
    }

    let db = Database::setup_for_testing::<OptionMapTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Test with Some value
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());

    let data_some = OptionMapTestData {
        required: OptionMapPayload {
            data: map.clone(),
        },
        optional: Some(OptionMapPayload {
            data: map.clone(),
        }),
    };

    query!(&mut conn, INSERT INTO OptionMapTestTable VALUES {data_some}).await?;

    let row: OptionMapTestData = query!(&mut conn,
        SELECT OptionMapTestData FROM OptionMapTestTable WHERE OptionMapTestTable.id = 1
    )
    .await?;

    assert!(row.optional.is_some());
    assert_eq!(row.optional.unwrap().data.get("key"), Some(&"value".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with Option<Vec> - Some(empty) vs None distinction
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_option_vec_empty_vs_none() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct OptionVecPayload {
        items: Vec<i32>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct OptionVecTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        optional_vec: Option<OptionVecPayload>,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = OptionVecTestTable)]
    #[sql(default = id)]
    struct OptionVecTestData {
        #[sql(bytes)]
        optional_vec: Option<OptionVecPayload>,
    }

    let db = Database::setup_for_testing::<OptionVecTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert with Some(empty_vec)
    let data_empty = OptionVecTestData {
        optional_vec: Some(OptionVecPayload { items: vec![] }),
    };

    query!(&mut conn, INSERT INTO OptionVecTestTable VALUES {data_empty}).await?;

    let row_empty: OptionVecTestData = query!(&mut conn,
        SELECT OptionVecTestData FROM OptionVecTestTable WHERE OptionVecTestTable.id = 1
    )
    .await?;

    assert!(row_empty.optional_vec.is_some());
    assert!(row_empty.optional_vec.unwrap().items.is_empty());

    // Insert with None
    let data_none = OptionVecTestData {
        optional_vec: None,
    };

    query!(&mut conn, INSERT INTO OptionVecTestTable VALUES {data_none}).await?;

    let row_none: OptionVecTestData = query!(&mut conn,
        SELECT OptionVecTestData FROM OptionVecTestTable WHERE OptionVecTestTable.id = 2
    )
    .await?;

    assert!(row_none.optional_vec.is_none());

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.5 ADVANCED BYTES TESTS - Multiple Bytes Fields
// ==============================================

/// Test struct with multiple different bytes fields
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_multiple_different_types() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct MultiBytesPayload {
        map_field: HashMap<String, i32>,
        vec_field: Vec<String>,
        nested_field: Vec<u8>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct MultiBytesTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        payload1: MultiBytesPayload,
        #[sql(bytes)]
        payload2: MultiBytesPayload,
        #[sql(bytes)]
        payload3: MultiBytesPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = MultiBytesTestTable)]
    #[sql(default = id)]
    struct MultiBytesTestData {
        #[sql(bytes)]
        payload1: MultiBytesPayload,
        #[sql(bytes)]
        payload2: MultiBytesPayload,
        #[sql(bytes)]
        payload3: MultiBytesPayload,
    }

    let db = Database::setup_for_testing::<MultiBytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut map = HashMap::new();
    map.insert("a".to_string(), 1);

    let data = MultiBytesTestData {
        payload1: MultiBytesPayload {
            map_field: map.clone(),
            vec_field: vec!["one".to_string()],
            nested_field: vec![1, 2, 3],
        },
        payload2: MultiBytesPayload {
            map_field: map.clone(),
            vec_field: vec!["two".to_string()],
            nested_field: vec![4, 5, 6],
        },
        payload3: MultiBytesPayload {
            map_field: map.clone(),
            vec_field: vec!["three".to_string()],
            nested_field: vec![7, 8, 9],
        },
    };

    query!(&mut conn, INSERT INTO MultiBytesTestTable VALUES {data}).await?;

    let row: MultiBytesTestData = query!(&mut conn,
        SELECT MultiBytesTestData FROM MultiBytesTestTable WHERE MultiBytesTestTable.id = 1
    )
    .await?;

    assert_eq!(row.payload1.vec_field[0], "one");
    assert_eq!(row.payload2.vec_field[0], "two");
    assert_eq!(row.payload3.vec_field[0], "three");
    assert_eq!(row.payload1.nested_field, vec![1, 2, 3]);

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with RETURNING clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let updated_data = expr_test_data(99, "updated", false, None);
    let returned: ExprTestData = query!(&mut conn,
        UPDATE ExprTestTable SET {updated_data} WHERE id = 1 RETURNING ExprTestData
    )
    .await?;

    assert_eq!(returned.int_field, 99);
    assert_eq!(returned.str_field, "updated");

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with no matching rows
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let updated_data = expr_test_data(99, "updated", false, None);
    query!(&mut conn, UPDATE ExprTestTable SET {updated_data} WHERE id = 99999).await?;

    // Original data should remain unchanged
    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;

    assert_eq!(result.int_field, 42); // Original value

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with complex WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_complex_where() -> anyhow::Result<()> {
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

    let updated_data = expr_test_data(100, "updated", true, None);
    query!(&mut conn,
        UPDATE ExprTestTable SET {updated_data}
        WHERE str_field = "test" AND bool_field = true
    )
    .await?;

    // Only first row should be updated
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field = 100
    )
    .await?;

    assert_eq!(results.len(), 1);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.6 ADVANCED BYTES TESTS - Maybe Update Patterns
// ==============================================

/// Test maybe_update with bytes HashMap field
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_maybe_update_hashmap() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct ConfigPayload {
        settings: HashMap<String, String>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct ConfigTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        config: ConfigPayload,
        #[sql(bytes)]
        metadata: Option<ConfigPayload>,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = ConfigTestTable)]
    #[sql(default = id)]
    struct ConfigTestData {
        #[sql(bytes)]
        config: ConfigPayload,
        #[sql(bytes)]
        metadata: Option<ConfigPayload>,
    }

    let db = Database::setup_for_testing::<ConfigTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut initial_settings = HashMap::new();
    initial_settings.insert("theme".to_string(), "dark".to_string());
    initial_settings.insert("language".to_string(), "en".to_string());

    let data = ConfigTestData {
        config: ConfigPayload {
            settings: initial_settings,
        },
        metadata: Some(ConfigPayload {
            settings: HashMap::new(),
        }),
    };

    query!(&mut conn, INSERT INTO ConfigTestTable VALUES {data}).await?;

    // Update with maybe_update on metadata
    #[derive(Update)]
    #[sql(table = ConfigTestTable)]
    struct ConfigUpdate {
        #[sql(bytes)]
        config: ConfigPayload,
        #[sql(bytes)]
        #[sql(maybe_update)]
        metadata: Option<Option<ConfigPayload>>,
    }

    let mut new_metadata = HashMap::new();
    new_metadata.insert("version".to_string(), "1.0".to_string());

    let update_data = ConfigUpdate {
        config: ConfigPayload {
            settings: HashMap::new(),
        },
        metadata: Some(Some(ConfigPayload {
            settings: new_metadata,
        })),
    };

    query!(&mut conn,
        UPDATE ConfigTestTable SET {update_data} WHERE ConfigTestTable.id = 1
    )
    .await?;

    let row: ConfigTestData = query!(&mut conn,
        SELECT ConfigTestData FROM ConfigTestTable WHERE ConfigTestTable.id = 1
    )
    .await?;

    assert!(row.metadata.is_some());

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update changing Some to None
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_maybe_update_some_to_none() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct DataPayload {
        value: String,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct MaybeUpdateTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        required: DataPayload,
        #[sql(bytes)]
        optional: Option<DataPayload>,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = MaybeUpdateTestTable)]
    #[sql(default = id)]
    struct MaybeUpdateTestData {
        #[sql(bytes)]
        required: DataPayload,
        #[sql(bytes)]
        optional: Option<DataPayload>,
    }

    let db = Database::setup_for_testing::<MaybeUpdateTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = MaybeUpdateTestData {
        required: DataPayload {
            value: "initial".to_string(),
        },
        optional: Some(DataPayload {
            value: "to_be_removed".to_string(),
        }),
    };

    query!(&mut conn, INSERT INTO MaybeUpdateTestTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = MaybeUpdateTestTable)]
    struct MaybeUpdateData {
        #[sql(bytes)]
        required: DataPayload,
        #[sql(bytes)]
        #[sql(maybe_update)]
        optional: Option<Option<DataPayload>>,
    }

    let update_data = MaybeUpdateData {
        required: DataPayload {
            value: "updated".to_string(),
        },
        optional: Some(None), // Change Some to None
    };

    query!(&mut conn,
        UPDATE MaybeUpdateTestTable SET {update_data} WHERE MaybeUpdateTestTable.id = 1
    )
    .await?;

    let row: MaybeUpdateTestData = query!(&mut conn,
        SELECT MaybeUpdateTestData FROM MaybeUpdateTestTable WHERE MaybeUpdateTestTable.id = 1
    )
    .await?;

    assert_eq!(row.required.value, "updated");
    assert!(row.optional.is_none());

    conn.rollback().await?;
    Ok(())
}

/// Test maybe_update changing to empty collection
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_maybe_update_to_empty() -> anyhow::Result<()> {
    use std::collections::HashMap;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct EmptyTestPayload {
        data: HashMap<String, i32>,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct EmptyMaybeTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        payload: Option<EmptyTestPayload>,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = EmptyMaybeTestTable)]
    #[sql(default = id)]
    struct EmptyMaybeTestData {
        #[sql(bytes)]
        payload: Option<EmptyTestPayload>,
    }

    let db = Database::setup_for_testing::<EmptyMaybeTestTable>().await?;
    let mut conn = db.transaction().await?;

    let mut initial_data = HashMap::new();
    initial_data.insert("key1".to_string(), 1);
    initial_data.insert("key2".to_string(), 2);

    let data = EmptyMaybeTestData {
        payload: Some(EmptyTestPayload {
            data: initial_data,
        }),
    };

    query!(&mut conn, INSERT INTO EmptyMaybeTestTable VALUES {data}).await?;

    #[derive(Update)]
    #[sql(table = EmptyMaybeTestTable)]
    struct EmptyUpdatePayload {
        #[sql(bytes)]
        #[sql(maybe_update)]
        payload: Option<Option<EmptyTestPayload>>,
    }

    // Update to empty HashMap
    let update_data = EmptyUpdatePayload {
        payload: Some(Some(EmptyTestPayload {
            data: HashMap::new(),
        })),
    };

    query!(&mut conn,
        UPDATE EmptyMaybeTestTable SET {update_data} WHERE EmptyMaybeTestTable.id = 1
    )
    .await?;

    let row: EmptyMaybeTestData = query!(&mut conn,
        SELECT EmptyMaybeTestData FROM EmptyMaybeTestTable WHERE EmptyMaybeTestTable.id = 1
    )
    .await?;

    assert!(row.payload.is_some());
    assert!(row.payload.unwrap().data.is_empty());

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.7 ADVANCED BYTES TESTS - Custom Serialize/Deserialize
// ==============================================

/// Test bytes with custom serialize/deserialize using serde_with-style
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_custom_struct_with_derive() -> anyhow::Result<()> {
    // Custom struct with derived Serialize/Deserialize that transforms data
    #[derive(Debug, Clone, PartialEq)]
    struct UppercaseString(String);

    impl serde::Serialize for UppercaseString {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.0.to_uppercase())
        }
    }

    impl<'de> serde::Deserialize<'de> for UppercaseString {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            Ok(UppercaseString(s))
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct CustomPayload {
        name: UppercaseString,
        count: i32,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct CustomTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: CustomPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = CustomTestTable)]
    #[sql(default = id)]
    struct CustomTestData {
        #[sql(bytes)]
        data: CustomPayload,
    }

    let db = Database::setup_for_testing::<CustomTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CustomTestData {
        data: CustomPayload {
            name: UppercaseString("hello".to_string()),
            count: 42,
        },
    };

    query!(&mut conn, INSERT INTO CustomTestTable VALUES {data}).await?;

    let row: CustomTestData = query!(&mut conn,
        SELECT CustomTestData FROM CustomTestTable WHERE CustomTestTable.id = 1
    )
    .await?;

    // The custom serialize converts to uppercase, deserialize keeps as-is
    assert_eq!(row.data.count, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test bytes with struct that has default values on deserialize
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_deserialize_with_defaults() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct DefaultFieldsPayload {
        #[serde(default = "default_name")]
        name: String,
        #[serde(default = "default_count")]
        count: i32,
    }

    fn default_name() -> String {
        "default".to_string()
    }

    fn default_count() -> i32 {
        100
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct DefaultFieldsTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        #[sql(bytes)]
        data: DefaultFieldsPayload,
    }

    #[derive(Insert, Output, Debug, Clone, PartialEq)]
    #[sql(table = DefaultFieldsTestTable)]
    #[sql(default = id)]
    struct DefaultFieldsTestData {
        #[sql(bytes)]
        data: DefaultFieldsPayload,
    }

    let db = Database::setup_for_testing::<DefaultFieldsTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = DefaultFieldsTestData {
        data: DefaultFieldsPayload {
            name: "custom".to_string(),
            count: 50,
        },
    };

    query!(&mut conn, INSERT INTO DefaultFieldsTestTable VALUES {data}).await?;

    let row: DefaultFieldsTestData = query!(&mut conn,
        SELECT DefaultFieldsTestData FROM DefaultFieldsTestTable WHERE DefaultFieldsTestTable.id = 1
    )
    .await?;

    assert_eq!(row.data.name, "custom");
    assert_eq!(row.data.count, 50);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3.8 ADVANCED BYTES TESTS - Bytes with Other SQL Attributes
// ==============================================

/// Test bytes with custom select expression
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_bytes_with_custom_select() -> anyhow::Result<()> {
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct SelectPayload {
        value: String,
    }

    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    struct SelectBytesTestTable {
        #[sql(primary_key)]
        id: String,
        #[sql(bytes)]
        data: SelectPayload,
    }

    #[derive(Insert, Debug, Clone, PartialEq)]
    #[sql(table = SelectBytesTestTable)]
    struct SelectBytesInsertData {
        id: String,
        #[sql(bytes)]
        data: SelectPayload,
    }

    #[derive(Output, Debug, Clone, PartialEq)]
    #[sql(table = SelectBytesTestTable)]
    struct SelectBytesOutputData {
        #[sql(bytes)]
        data: SelectPayload,
    }

    let db = Database::setup_for_testing::<SelectBytesTestTable>().await?;
    let mut conn = db.transaction().await?;

    let insert_data = SelectBytesInsertData {
        id: "test_id".to_string(),
        data: SelectPayload {
            value: "test_value".to_string(),
        },
    };

    query!(&mut conn, INSERT INTO SelectBytesTestTable VALUES {insert_data}).await?;

    let row: SelectBytesOutputData = query!(&mut conn,
        SELECT SelectBytesOutputData FROM SelectBytesTestTable WHERE SelectBytesTestTable.id = "test_id"
    )
    .await?;

    assert_eq!(row.data.value, "test_value");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 4. DELETE QUERIES
// ==============================================

/// Test simple DELETE
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_single() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    query!(&mut conn, DELETE FROM ExprTestTable WHERE id = 1).await?;

    // Verify deletion
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE multiple rows
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_multiple() -> anyhow::Result<()> {
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

    query!(&mut conn, DELETE FROM ExprTestTable WHERE str_field = "test").await?;

    // Verify only "other" remains
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].str_field, "other");

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with RETURNING clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_returning() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let returned: ExprTestData = query!(&mut conn,
        DELETE FROM ExprTestTable WHERE id = 1 RETURNING ExprTestData
    )
    .await?;

    assert_eq!(returned.int_field, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with no matching rows
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_no_match() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    query!(&mut conn, DELETE FROM ExprTestTable WHERE id = 99999).await?;

    // Original data should remain
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with complex WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_complex_where() -> anyhow::Result<()> {
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

    query!(&mut conn,
        DELETE FROM ExprTestTable
        WHERE int_field BETWEEN 15 AND 35 AND str_field = "test"
    )
    .await?;

    // Should delete only the second row
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with variable in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_variable_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "first", true, None),
            expr_test_data(20, "second", false, None),
            expr_test_data(30, "third", true, None),
        ],
    )
    .await?;

    let target_id = 2;
    let updated = expr_test_data(99, "updated", true, None);

    query!(&mut conn,
        UPDATE ExprTestTable SET {updated} WHERE id = {target_id}
    )
    .await?;

    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 2
    )
    .await?;

    assert_eq!(result.int_field, 99);
    assert_eq!(result.str_field, "updated");
    assert!(result.bool_field);

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with multiple variables in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_multiple_variable_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "apple", true, None),
            expr_test_data(20, "banana", false, None),
            expr_test_data(30, "apple", true, None),
            expr_test_data(40, "cherry", false, None),
        ],
    )
    .await?;

    let search_str = "apple".to_string();
    let search_bool = true;
    let updated = expr_test_data(777, "modified", false, Some("data"));

    query!(&mut conn,
        UPDATE ExprTestTable SET {updated}
        WHERE str_field = {search_str} AND bool_field = {search_bool}
    )
    .await?;

    // Should update both apple + true records (rows 1 and 3)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field = 777
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].str_field, "modified");
    assert_eq!(results[1].str_field, "modified");

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with BETWEEN variable in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_between_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let min_val = 15;
    let max_val = 35;
    let updated = expr_test_data(555, "range_update", false, None);

    query!(&mut conn,
        UPDATE ExprTestTable SET {updated}
        WHERE int_field BETWEEN {min_val} AND {max_val}
    )
    .await?;

    // Should update rows 2 and 3 (int_field 20 and 30)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE str_field = "range_update"
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 555);
    assert_eq!(results[1].int_field, 555);

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with IN operator and variables
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_with_in_operator_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
            expr_test_data(50, "e", true, None),
        ],
    )
    .await?;

    let target_id = 2;
    let updated = expr_test_data(888, "in_update", false, None);

    query!(&mut conn,
        UPDATE ExprTestTable SET {updated}
        WHERE id IN (1, {target_id}, 4)
    )
    .await?;

    // Should update rows 1, 2, and 4
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field = 888
    )
    .await?;

    assert_eq!(results.len(), 3);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with variable in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_variable_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "first", true, None),
            expr_test_data(20, "second", false, None),
            expr_test_data(30, "third", true, None),
        ],
    )
    .await?;

    let target_id = 2;

    query!(&mut conn,
        DELETE FROM ExprTestTable WHERE id = {target_id}
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with multiple variables in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_multiple_variable_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "apple", true, None),
            expr_test_data(20, "banana", false, None),
            expr_test_data(30, "apple", true, None),
            expr_test_data(40, "cherry", false, None),
        ],
    )
    .await?;

    let search_str = "apple".to_string();
    let search_bool = true;

    query!(&mut conn,
        DELETE FROM ExprTestTable
        WHERE str_field = {search_str} AND bool_field = {search_bool}
    )
    .await?;

    // Should delete rows 1 and 3
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with BETWEEN variable in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_between_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let min_val = 18;
    let max_val = 32;

    query!(&mut conn,
        DELETE FROM ExprTestTable
        WHERE int_field BETWEEN {min_val} AND {max_val}
    )
    .await?;

    // Should delete rows 2 and 3
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with LIKE pattern variable
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_like_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "test_one", true, None),
            expr_test_data(20, "test_two", true, None),
            expr_test_data(30, "other", true, None),
        ],
    )
    .await?;

    let pattern = "test%".to_string();

    query!(&mut conn,
        DELETE FROM ExprTestTable
        WHERE str_field LIKE {pattern}
    )
    .await?;

    // Should delete rows 1 and 2
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 30);
    assert_eq!(results[0].str_field, "other");

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with IN operator and variables
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_with_in_operator_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
            expr_test_data(50, "e", true, None),
        ],
    )
    .await?;

    let target_val = 30;

    query!(&mut conn,
        DELETE FROM ExprTestTable
        WHERE int_field IN (10, {target_val}, 50)
    )
    .await?;

    // Should delete rows 1, 3, and 5
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with variables in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_variable_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "test", true, None),
            expr_test_data(20, "other", false, None),
        ],
    )
    .await?;

    let search_str = "test".to_string();
    let search_bool = true;

    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE str_field = {search_str} AND bool_field = {search_bool}
    )
    .await?;

    assert!(exists);

    let not_exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE str_field = {search_str} AND bool_field = false
    )
    .await?;

    assert!(!not_exists);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 5. EXISTS QUERIES
// ==============================================

/// Test EXISTS returns true
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_true() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let exists: bool = query!(&mut conn, EXISTS ExprTestTable WHERE id = 1).await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS returns false
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_false() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let exists: bool = query!(&mut conn, EXISTS ExprTestTable WHERE id = 99999).await?;

    assert!(!exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with complex WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_complex_where() -> anyhow::Result<()> {
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

    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 15 AND str_field = "test" AND bool_field = false
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS on empty table
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_empty_table() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let exists: bool = query!(&mut conn, EXISTS ExprTestTable WHERE true).await?;

    assert!(!exists);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 6. INTEGRATION TESTS (CRUD workflows)
// ==============================================

/// Test complete CRUD lifecycle
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_crud_lifecycle() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // CREATE (INSERT)
    let data = default_expr_test_data();
    query!(&mut conn, INSERT INTO ExprTestTable VALUES {data}).await?;

    // READ (SELECT)
    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;
    assert_eq!(result.int_field, 42);

    // UPDATE
    let updated_data = expr_test_data(99, "updated", false, None);
    query!(&mut conn, UPDATE ExprTestTable SET {updated_data} WHERE id = 1).await?;

    // READ updated
    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;
    assert_eq!(result.int_field, 99);

    // DELETE
    query!(&mut conn, DELETE FROM ExprTestTable WHERE id = 1).await?;

    // Verify deletion
    let exists: bool = query!(&mut conn, EXISTS ExprTestTable WHERE id = 1).await?;
    assert!(!exists);

    conn.rollback().await?;
    Ok(())
}

/// Test multiple operations in single transaction
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_transaction_isolation() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert multiple records
    for i in 1..=5 {
        let data = expr_test_data(i * 10, &format!("test{}", i), true, None);
        query!(&mut conn, INSERT INTO ExprTestTable VALUES {data}).await?;
    }

    // Query within transaction
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field > 20
    )
    .await?;

    assert_eq!(results.len(), 3);

    // Rollback - data should not persist
    conn.rollback().await?;

    // Verify rollback (new connection)
    let mut conn2 = db.conn().await?;
    let exists: bool = query!(&mut conn2, EXISTS ExprTestTable WHERE true).await?;
    assert!(!exists);

    Ok(())
}

// ==============================================
// 7. NEW FEATURE TESTS
// ==============================================

// ====================
// 7.1 EXISTS with new clauses
// ====================

/// Test EXISTS with GROUP BY clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_group_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "group_a", true, None),
            expr_test_data(20, "group_a", true, None),
            expr_test_data(30, "group_b", true, None),
        ],
    )
    .await?;

    // Test EXISTS with GROUP BY
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        GROUP BY str_field
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with GROUP BY and HAVING clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_having() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "group_a", true, None),
            expr_test_data(20, "group_a", true, None),
            expr_test_data(30, "group_a", true, None),
            expr_test_data(40, "group_b", true, None),
        ],
    )
    .await?;

    // EXISTS with GROUP BY and HAVING (using column comparison)
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        GROUP BY str_field
        HAVING str_field = "group_a"
    )
    .await?;

    assert!(exists);

    // Test HAVING with condition that doesn't match
    let not_exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 0
        GROUP BY str_field
        HAVING str_field = "nonexistent"
    )
    .await?;

    assert!(!not_exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with HAVING and variable
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_having_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "group_a", true, None),
            expr_test_data(20, "group_a", true, None),
            expr_test_data(30, "group_b", true, None),
        ],
    )
    .await?;

    let group_name = "group_a";
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 0
        GROUP BY str_field
        HAVING str_field = {group_name}
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with ORDER BY clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_order_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(30, "c", true, None),
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
        ],
    )
    .await?;

    // EXISTS with ORDER BY (mostly for syntax validation)
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        ORDER BY int_field DESC
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with LIMIT clause (literal)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_limit_literal() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    // EXISTS with LIMIT 1 (performance optimization)
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        LIMIT 1
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with LIMIT clause (variable)
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_limit_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    let limit_val = 1;
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        LIMIT {limit_val}
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with OFFSET clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_offset() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    let offset_with_remaining_rows = 2;
    let exists_with_offset: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        ORDER BY int_field ASC
        LIMIT 10
        OFFSET {offset_with_remaining_rows}
    )
    .await?;
    assert!(exists_with_offset);

    let offset_without_remaining_rows = 3;
    let exists_without_rows_after_offset: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        ORDER BY int_field ASC
        LIMIT 10
        OFFSET {offset_without_remaining_rows}
    )
    .await?;
    assert!(!exists_without_rows_after_offset);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS accepts OFFSET before LIMIT in macro input
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_offset_before_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    let offset_with_remaining_rows = 2;
    let exists_with_offset: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        ORDER BY int_field ASC
        OFFSET {offset_with_remaining_rows}
        LIMIT 10
    )
    .await?;
    assert!(exists_with_offset);

    let offset_without_remaining_rows = 3;
    let exists_without_rows_after_offset: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        ORDER BY int_field ASC
        OFFSET {offset_without_remaining_rows}
        LIMIT 10
    )
    .await?;
    assert!(!exists_without_rows_after_offset);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with all clauses combined
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_with_all_clauses() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "group_a", true, None),
            expr_test_data(20, "group_a", true, None),
            expr_test_data(30, "group_a", true, None),
            expr_test_data(40, "group_b", true, None),
            expr_test_data(50, "group_b", true, None),
        ],
    )
    .await?;

    let min_count = 2;
    let limit_val = 1;

    // EXISTS with WHERE, GROUP BY, HAVING, ORDER BY, LIMIT
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field > 5
        GROUP BY str_field
        HAVING COUNT(*) >= {min_count}
        ORDER BY COUNT(*) DESC
        LIMIT {limit_val}
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

// ====================
// 7.2 LIMIT clause with variables in SELECT
// ====================

/// Test SELECT with LIMIT variable
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_limit_with_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
        ],
    )
    .await?;

    let limit_value = 2;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE true
        LIMIT {limit_value}
    )
    .await?;

    assert_eq!(results.len(), 2);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with dynamic LIMIT based on condition
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_dynamic_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
            expr_test_data(50, "e", true, None),
        ],
    )
    .await?;

    // Simulate pagination with dynamic limit
    let page_size = 3;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field > 0
        ORDER BY int_field
        LIMIT {page_size}
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test SELECT with LIMIT and WHERE variables
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_limit_and_where_variables() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", true, None),
            expr_test_data(50, "e", true, None),
        ],
    )
    .await?;

    let min_val = 15;
    let max_results = 2;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field > {min_val}
        ORDER BY int_field
        LIMIT {max_results}
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

// ====================
// 7.3 Multiple IN clauses with variables
// ====================
//
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_where_multiple_in_clauses() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
            expr_test_data(40, "delta", false, None),
            expr_test_data(50, "epsilon", true, None),
        ],
    )
    .await?;

    let int_values = vec![10, 30, 50];
    let str_values = vec!["alpha", "gamma", "epsilon"];

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN {int_values} AND str_field IN {str_values}
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);
    assert_eq!(results[2].int_field, 50);

    conn.rollback().await?;
    Ok(())
}

/// Test WHERE with multiple IN clauses and additional conditions
///
/// ❌ NOT SUPPORTED: IN with variables - see MISSING_FEATURES_REPORT.md Section 1
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_where_multiple_in_with_conditions() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
            expr_test_data(40, "delta", false, None),
            expr_test_data(50, "epsilon", true, None),
        ],
    )
    .await?;

    let int_values = vec![10, 20, 30, 40, 50];
    let str_values = vec!["alpha", "beta", "gamma"];

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN {int_values}
            AND str_field IN {str_values}
            AND bool_field = true
    )
    .await?;

    assert_eq!(results.len(), 2); // alpha and gamma
    assert_eq!(results[0].str_field, "alpha");
    assert_eq!(results[1].str_field, "gamma");

    conn.rollback().await?;
    Ok(())
}

/// Test WHERE with multiple IN clauses and OR
///
/// ❌ NOT SUPPORTED: IN with variables - see MISSING_FEATURES_REPORT.md Section 1
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_where_multiple_in_with_or() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
            expr_test_data(40, "delta", false, None),
        ],
    )
    .await?;

    let low_values = vec![10, 20];
    let high_values = vec![30, 40];

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN {low_values} OR int_field IN {high_values}
    )
    .await?;

    assert_eq!(results.len(), 4);

    conn.rollback().await?;
    Ok(())
}

/// Test EXISTS with multiple IN clauses
///
/// ❌ NOT SUPPORTED: IN with variables - see MISSING_FEATURES_REPORT.md Section 1
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_exists_multiple_in_clauses() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
        ],
    )
    .await?;

    let int_values = vec![10, 30];
    let str_values = vec!["alpha", "gamma"];

    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable
        WHERE int_field IN {int_values} AND str_field IN {str_values}
    )
    .await?;

    assert!(exists);

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with multiple IN clauses
///
/// ❌ NOT SUPPORTED: IN with variables - see MISSING_FEATURES_REPORT.md Section 1
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_multiple_in_clauses() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
            expr_test_data(40, "delta", false, None),
        ],
    )
    .await?;

    let int_values = vec![10, 30];
    let str_values = vec!["alpha", "gamma"];

    query!(&mut conn,
        UPDATE ExprTestTable
        SET bool_field = false
        WHERE int_field IN {&int_values} AND str_field IN {&str_values}
    )
    .await?;

    // Verify update
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN {int_values} AND str_field IN {str_values}
    )
    .await?;

    assert_eq!(results.len(), 2);
    for result in results {
        assert!(!result.bool_field);
    }

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with multiple IN clauses
///
/// ❌ NOT SUPPORTED: IN with variables - see MISSING_FEATURES_REPORT.md Section 1
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_multiple_in_clauses() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
            expr_test_data(40, "delta", false, None),
        ],
    )
    .await?;

    let int_values = vec![10, 30];
    let str_values = vec!["alpha", "gamma"];

    query!(&mut conn,
        DELETE FROM ExprTestTable
        WHERE int_field IN {int_values} AND str_field IN {str_values}
    )
    .await?;

    // Verify deletion
    let remaining: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE true
    )
    .await?;

    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0].str_field, "beta");
    assert_eq!(remaining[1].str_field, "delta");

    conn.rollback().await?;
    Ok(())
}

/// Test complex query with multiple IN clauses, LIMIT, and parameter binding
///
/// ❌ NOT SUPPORTED: IN with variables - see MISSING_FEATURES_REPORT.md Section 1
#[always_context(skip(!))]
#[tokio::test]
async fn test_query_complex_multiple_in_with_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "alpha", true, None),
            expr_test_data(20, "beta", false, None),
            expr_test_data(30, "gamma", true, None),
            expr_test_data(40, "delta", false, None),
            expr_test_data(50, "epsilon", true, None),
        ],
    )
    .await?;

    let int_values = vec![10, 20, 30, 40, 50];
    let str_values = vec!["alpha", "gamma", "epsilon"];
    let max_results = 2;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN {int_values}
            AND str_field IN {str_values}
            AND bool_field = true
        ORDER BY int_field
        LIMIT {max_results}
    )
    .await
    .context("")?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}
