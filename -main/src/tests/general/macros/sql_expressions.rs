// Comprehensive tests for SQL expressions (Expr enum)
// Tests all operators, comparisons, and expression types

use super::*;
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;
#[cfg(feature = "postgres")]
use sqlx::types::JsonValue;

// ==============================================
// JSON TEST TABLES (PostgreSQL-only)
// ==============================================

#[cfg(feature = "postgres")]
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct JsonOperatorTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    payload: JsonValue,
}

#[cfg(feature = "postgres")]
#[derive(Insert, Output, Debug, Clone, PartialEq)]
#[sql(table = JsonOperatorTable)]
#[sql(default = id)]
struct JsonOperatorData {
    payload: JsonValue,
}

// ==============================================
// 1. COMPARISON OPERATORS
// ==============================================

/// Test basic equality with literal value
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_equal_with_literal() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let result: ExprTestData =
        query!(&mut conn, SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1)
            .await?;

    assert_eq!(result.int_field, 42);
    assert_eq!(result.str_field, "test");

    conn.rollback().await?;
    Ok(())
}

/// Test equality with variable binding
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_equal_with_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let search_id = 1;
    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = {search_id}
    )
    .await?;

    assert_eq!(result.int_field, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test not equal operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_not_equal() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "test1", true, None),
            expr_test_data(20, "test2", false, None),
            expr_test_data(30, "test3", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field != 20
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test greater than operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_greater_than() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field > 15
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test greater than or equal operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_greater_than_or_equal() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field >= 20
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test less than operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_less_than() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field < 25
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test less than or equal operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_less_than_or_equal() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE int_field <= 20
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test string equality comparison
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_string_comparison() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "apple", true, None),
            expr_test_data(2, "banana", true, None),
            expr_test_data(3, "cherry", true, None),
        ],
    )
    .await?;

    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE str_field = "banana"
    )
    .await?;

    assert_eq!(result.int_field, 2);
    assert_eq!(result.str_field, "banana");

    conn.rollback().await?;
    Ok(())
}

/// Test boolean field comparison
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_boolean_comparison() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "a", true, None),
            expr_test_data(2, "b", false, None),
            expr_test_data(3, "c", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE bool_field = true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 1);
    assert_eq!(results[1].int_field, 3);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 2. LOGICAL OPERATORS
// ==============================================

/// Test simple AND operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_and_operator() -> anyhow::Result<()> {
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
        WHERE int_field > 5 AND str_field = "test"
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test simple OR operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_or_operator() -> anyhow::Result<()> {
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
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field = 10 OR int_field = 30
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test NOT operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_not_operator() -> anyhow::Result<()> {
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
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE NOT int_field = 20
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test chained AND conditions
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_chained_and() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "test", true, Some("data")),
            expr_test_data(20, "test", true, None),
            expr_test_data(30, "test", false, Some("data")),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field = "test" AND bool_field = true AND int_field > 5
    )
    .await?;

    assert_eq!(results.len(), 2);

    conn.rollback().await?;
    Ok(())
}

/// Test parenthesized expressions
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_parenthesized() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", false, None),
        ],
    )
    .await?;

    // (int_field = 10 OR int_field = 30) AND bool_field = true
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (int_field = 10 OR int_field = 30) AND bool_field = true
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test mixed AND/OR without parentheses (operator precedence)
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_mixed_and_or() -> anyhow::Result<()> {
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

    // Should be: (int_field = 10) OR (str_field = "test" AND bool_field = false)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field = 10 OR str_field = "test" AND bool_field = false
    )
    .await?;

    assert_eq!(results.len(), 2); // Records 1 and 2

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3. STRING OPERATORS
// ==============================================

/// Test LIKE operator with pattern
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_like_operator() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "test_one", true, None),
            expr_test_data(2, "test_two", true, None),
            expr_test_data(3, "other", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field LIKE "test%"
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert!(results[0].str_field.starts_with("test"));
    assert!(results[1].str_field.starts_with("test"));

    conn.rollback().await?;
    Ok(())
}

/// Test LIKE with variable pattern
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_like_with_variable() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "hello_world", true, None),
            expr_test_data(2, "hello_there", true, None),
            expr_test_data(3, "goodbye", true, None),
        ],
    )
    .await?;

    let pattern = "hello%".to_string();
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field LIKE {pattern}
    )
    .await?;

    assert_eq!(results.len(), 2);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 4. NULL HANDLING
// ==============================================

/// Test IS NULL operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_is_null() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "a", true, None),
            expr_test_data(2, "b", true, Some("data")),
            expr_test_data(3, "c", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE nullable_field IS NULL
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert!(results[0].nullable_field.is_none());
    assert!(results[1].nullable_field.is_none());

    conn.rollback().await?;
    Ok(())
}

/// Test IS NOT NULL operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_is_not_null() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "a", true, None),
            expr_test_data(2, "b", true, Some("data")),
            expr_test_data(3, "c", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE nullable_field IS NOT NULL
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].nullable_field, Some("data".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test NULL in complex condition
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_null_in_complex_condition() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, Some("data")),
            expr_test_data(30, "c", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field > 5 AND nullable_field IS NULL
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 5. IN OPERATOR
// ==============================================

/// Test IN operator with multiple values
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_in_operator() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN (10, 30, 40)
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);
    assert_eq!(results[2].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test IN operator with single value (ValueIn::Multiple mode with one item)
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_in_single_value() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN (10)
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test IN operator with string values in parentheses (ValueIn::Multiple mode)
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_in_with_string_literals() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "apple", true, None),
            expr_test_data(2, "banana", true, None),
            expr_test_data(3, "cherry", true, None),
            expr_test_data(4, "date", true, None),
        ],
    )
    .await?;

    // Test with string literals (ValueIn::Multiple mode)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field IN ("apple", "cherry", "date")
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].str_field, "apple");
    assert_eq!(results[1].str_field, "cherry");
    assert_eq!(results[2].str_field, "date");

    conn.rollback().await?;
    Ok(())
}

/// Test IN operator with variables in parentheses (ValueIn::Multiple mode)
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_in_with_multiple_variables() -> anyhow::Result<()> {
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

    // Test with multiple variables in parentheses (ValueIn::Multiple mode)
    let val1 = 10;
    let val2 = 30;
    let val3 = 40;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN ({val1}, {val2}, {val3})
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);
    assert_eq!(results[2].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test NOT IN operator with multiple values
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_in_mixed_literals_and_variables() -> anyhow::Result<()> {
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

    // Test with mix of literals and variables
    let val = 30;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field IN (10, {val}, 40)
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);
    assert_eq!(results[2].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test NOT IN operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_not_in() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE NOT int_field IN (20)
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 6. BETWEEN OPERATOR
// ==============================================

/// Test BETWEEN operator with integers
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_between_integers() -> anyhow::Result<()> {
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

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field BETWEEN 15 AND 35
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test BETWEEN with variables
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_between_with_variables() -> anyhow::Result<()> {
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
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field BETWEEN {min_val} AND {max_val}
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test BETWEEN with strings (lexicographic order)
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_between_strings() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "apple", true, None),
            expr_test_data(2, "banana", true, None),
            expr_test_data(3, "cherry", true, None),
            expr_test_data(4, "date", true, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field BETWEEN "banana" AND "date"
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].str_field, "banana");
    assert_eq!(results[1].str_field, "cherry");
    assert_eq!(results[2].str_field, "date");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 7. COMPLEX NESTED EXPRESSIONS
// ==============================================

/// Test deeply nested AND/OR conditions
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_deeply_nested_conditions() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "test", true, None),
            expr_test_data(20, "test", false, None),
            expr_test_data(30, "other", true, None),
            expr_test_data(40, "other", false, Some("data")),
        ],
    )
    .await?;

    // ((int_field = 10 OR int_field = 30) AND bool_field = true) OR (str_field = "other" AND nullable_field IS NOT NULL)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE ((int_field = 10 OR int_field = 30) AND bool_field = true)
           OR (str_field = "other" AND nullable_field IS NOT NULL)
    )
    .await?;

    assert_eq!(results.len(), 3); // Records 1, 3, 4

    conn.rollback().await?;
    Ok(())
}

/// Test complex condition with all operator types
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_all_operators_mixed() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(15, "test_one", true, None),
            expr_test_data(25, "test_two", false, Some("data")),
            expr_test_data(35, "other", true, None),
        ],
    )
    .await?;

    // int_field BETWEEN 10 AND 30 AND str_field LIKE "test%" AND (bool_field = true OR nullable_field IS NOT NULL)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field BETWEEN 10 AND 30
          AND str_field LIKE "test%"
          AND (bool_field = true OR nullable_field IS NOT NULL)
    )
    .await?;

    assert_eq!(results.len(), 2); // Records 1 and 2

    conn.rollback().await?;
    Ok(())
}

/// Test multiple levels of parenthesization
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_multiple_parenthesis_levels() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(40, "d", false, None),
        ],
    )
    .await?;

    // (((int_field = 10) OR (int_field = 30)) AND (bool_field = true))
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (((int_field = 10) OR (int_field = 30)) AND (bool_field = true))
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 8. EDGE CASES
// ==============================================

/// Test empty result with Vec<T> return type
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_empty_result_vec() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field = 99999
    )
    .await?;

    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

/// Test query with always-true condition
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_always_true_condition() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
        ],
    )
    .await?;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2);

    conn.rollback().await?;
    Ok(())
}

/// Test with multiple variable bindings
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_multiple_variable_bindings() -> anyhow::Result<()> {
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

    let min_int = 15;
    let max_int = 35;
    let search_str = "test".to_string();
    let search_bool = false;

    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field >= {min_int}
          AND int_field <= {max_int}
          AND str_field = {search_str}
          AND bool_field = {search_bool}
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 9. ARITHMETIC OPERATORS (in WHERE clauses)
// ==============================================

/// Test addition operator in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_addition_operator() -> anyhow::Result<()> {
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

    // WHERE int_field + 5 = 25 should match int_field = 20
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field + 5 = 25
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test subtraction operator in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_subtraction_operator() -> anyhow::Result<()> {
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

    // WHERE int_field - 5 = 15 should match int_field = 20
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field - 5 = 15
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test multiplication operator in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_multiplication_operator() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(5, "a", true, None),
            expr_test_data(10, "b", true, None),
            expr_test_data(15, "c", true, None),
        ],
    )
    .await?;

    // WHERE int_field * 2 = 20 should match int_field = 10
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field * 2 = 20
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

/// Test division operator in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_division_operator() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(20, "a", true, None),
            expr_test_data(40, "b", true, None),
            expr_test_data(60, "c", true, None),
        ],
    )
    .await?;

    // WHERE int_field / 2 = 20 should match int_field = 40
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field / 2 = 20
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 40);

    conn.rollback().await?;
    Ok(())
}

/// Test modulo operator in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_modulo_operator() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(21, "b", true, None),
            expr_test_data(22, "c", true, None),
            expr_test_data(30, "d", true, None),
        ],
    )
    .await?;

    // WHERE int_field % 10 = 1 should match int_field = 21
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field % 10 = 1
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 21);

    conn.rollback().await?;
    Ok(())
}

/// Test chained arithmetic operations
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_chained_arithmetic() -> anyhow::Result<()> {
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

    // WHERE (int_field + 5) * 2 = 50 should match int_field = 20
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (int_field + 5) * 2 = 50
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test arithmetic with variables
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_arithmetic_with_variables() -> anyhow::Result<()> {
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

    let offset = 5;
    let target = 25;

    // WHERE int_field + {offset} = {target} should match int_field = 20
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field + {offset} = {target}
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 10. STRING CONCATENATION OPERATOR
// ==============================================

/// Test string concatenation operator in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_string_concat_postgres() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "hello", true, None),
            expr_test_data(2, "world", true, None),
            expr_test_data(3, "test", true, None),
        ],
    )
    .await?;

    // WHERE str_field || '_suffix' = 'hello_suffix'
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field || "_suffix" = "hello_suffix"
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].str_field, "hello");

    conn.rollback().await?;
    Ok(())
}

/// Test multiple string concatenations
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_multiple_string_concat() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "hello", true, None),
            expr_test_data(2, "world", true, None),
        ],
    )
    .await?;

    // WHERE str_field || '-' || 'test' = 'hello-test'
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE str_field || "-" || "test" = "hello-test"
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].str_field, "hello");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 11. BITWISE OPERATORS
// ==============================================

/// Test bitwise AND operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_bitwise_and() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(7, "a", true, None),  // 0111
            expr_test_data(12, "b", true, None), // 1100
            expr_test_data(15, "c", true, None), // 1111
        ],
    )
    .await?;

    // WHERE int_field & 4 = 4 (has bit 2 set)
    // Should match 7 (0111), 12 (1100), 15 (1111)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field & 4 = 4
    )
    .await?;

    assert_eq!(results.len(), 3);

    conn.rollback().await?;
    Ok(())
}

/// Test bitwise OR operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_bitwise_or() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(4, "a", true, None),  // 0100
            expr_test_data(8, "b", true, None),  // 1000
            expr_test_data(12, "c", true, None), // 1100
        ],
    )
    .await?;

    // WHERE int_field | 3 = 7 should match int_field = 4 (0100 | 0011 = 0111 = 7)
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field | 3 = 7
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 4);

    conn.rollback().await?;
    Ok(())
}

/// Test bitwise left shift operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_bitwise_left_shift() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(2, "a", true, None),
            expr_test_data(4, "b", true, None),
            expr_test_data(8, "c", true, None),
        ],
    )
    .await?;

    // WHERE int_field << 2 = 16 should match int_field = 4
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field << 2 = 16
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 4);

    conn.rollback().await?;
    Ok(())
}

/// Test bitwise right shift operator
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_bitwise_right_shift() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(16, "a", true, None),
            expr_test_data(32, "b", true, None),
            expr_test_data(64, "c", true, None),
        ],
    )
    .await?;

    // WHERE int_field >> 2 = 8 should match int_field = 32
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field >> 2 = 8
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 32);

    conn.rollback().await?;
    Ok(())
}

/// Test combined bitwise operations
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_combined_bitwise() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(12, "a", true, None), // 1100
            expr_test_data(15, "b", true, None), // 1111
            expr_test_data(10, "c", true, None), // 1010
        ],
    )
    .await?;

    // Test combining bitwise AND and OR: (int_field & 8) | 2 = 10
    // For value to equal 10:
    // 12: (12 & 8) | 2 = 8 | 2 = 10 ✓
    // 15: (15 & 8) | 2 = 8 | 2 = 10 ✓
    // 10: (10 & 8) | 2 = 8 | 2 = 10 ✓
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (int_field & 8) | 2 = 10
    )
    .await?;

    assert_eq!(results.len(), 3); // All three values should match

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 12. JSON OPERATORS (PostgreSQL-specific)
// ==============================================

/// Test JSON arrow operator (->) for accessing JSON field
#[cfg(feature = "postgres")]
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_json_arrow_operator() -> anyhow::Result<()> {
    use serde_json::json;

    let db = Database::setup_for_testing::<JsonOperatorTable>().await?;
    let mut conn = db.transaction().await?;

    let data = JsonOperatorData {
        payload: json!({"name": "Alice", "age": 30}),
    };
    query!(&mut conn, INSERT INTO JsonOperatorTable VALUES {data}).await?;

    let expected = json!("Alice");
    let result: JsonOperatorData = query!(&mut conn,
        SELECT JsonOperatorData FROM JsonOperatorTable
        WHERE payload -> "name" = {expected}
    )
    .await?;

    assert_eq!(result.payload.get("name").and_then(|v| v.as_str()), Some("Alice"));

    conn.rollback().await?;
    Ok(())
}

/// Test JSON double arrow operator (->>) for accessing JSON field as text
#[cfg(feature = "postgres")]
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_json_double_arrow_operator() -> anyhow::Result<()> {
    use serde_json::json;

    let db = Database::setup_for_testing::<JsonOperatorTable>().await?;
    let mut conn = db.transaction().await?;

    let data = JsonOperatorData {
        payload: json!({"name": "Ada", "active": true}),
    };
    query!(&mut conn, INSERT INTO JsonOperatorTable VALUES {data}).await?;

    let expected = "Ada".to_string();
    let result: JsonOperatorData = query!(&mut conn,
        SELECT JsonOperatorData FROM JsonOperatorTable
        WHERE payload ->> "name" = {expected}
    )
    .await?;

    assert_eq!(result.payload.get("name").and_then(|v| v.as_str()), Some("Ada"));

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 13. MIXED OPERATOR TYPES
// ==============================================

/// Test arithmetic combined with comparison
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_arithmetic_with_comparison() -> anyhow::Result<()> {
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

    // WHERE int_field * 2 > 35 should match 20 and 30
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE int_field * 2 > 35
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 20);
    assert_eq!(results[1].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

/// Test arithmetic combined with logical operators
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_arithmetic_with_logical() -> anyhow::Result<()> {
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

    // WHERE (int_field + 5 > 20) AND str_field = "test"
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (int_field + 5 > 20) AND str_field = "test"
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].int_field, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test bitwise combined with arithmetic
#[always_context(skip(!))]
#[tokio::test]
async fn test_expr_bitwise_with_arithmetic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(8, "a", true, None),
            expr_test_data(16, "b", true, None),
            expr_test_data(24, "c", true, None),
        ],
    )
    .await?;

    // WHERE (int_field & 8) + 2 = 10 should match values where bit 3 is set
    // 8 & 8 = 8, 8 + 2 = 10 ✓
    // 16 & 8 = 0, 0 + 2 = 2 ✗
    // 24 & 8 = 8, 8 + 2 = 10 ✓
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE (int_field & 8) + 2 = 10
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 8);
    assert_eq!(results[1].int_field, 24);

    conn.rollback().await?;
    Ok(())
}
