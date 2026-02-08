// Comprehensive tests for query! macro
// Tests all query types: SELECT, INSERT, UPDATE, DELETE, EXISTS

use super::*;
use anyhow::Context;
use easy_macros::{always_context /* always_context_debug as always_context */};
use serde::{Deserialize, Serialize};
use easy_sql_macros::query;

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
