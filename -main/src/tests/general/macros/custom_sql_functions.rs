// Tests for custom_sql_function! procedural macro

use super::*;
use easy_macros::always_context;
use easy_sql_macros::{custom_sql_function, query};

// ==============================================
// Define Custom Functions for Testing
// ==============================================

custom_sql_function!(FancyUpper; "UPPER"; 1);
custom_sql_function!(Slice; "SUBSTR"; 2 | 3);
custom_sql_function!(AnyCoalesce; "COALESCE"; Any);

// ==============================================
// Test Tables
// ==============================================

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct CustomFunctionTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub name: String,
    pub alt_name: Option<String>,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = CustomFunctionTestTable)]
#[sql(default = id)]
pub struct CustomFunctionTestData {
    pub name: String,
    pub alt_name: Option<String>,
}

// ==============================================
// Test Utilities
// ==============================================

fn test_data(name: &str, alt_name: Option<&str>) -> CustomFunctionTestData {
    CustomFunctionTestData {
        name: name.to_string(),
        alt_name: alt_name.map(|value| value.to_string()),
    }
}

#[always_context(skip(!))]
async fn setup_custom_test_data(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
) -> anyhow::Result<()> {
    let test_data = vec![test_data("hello", None), test_data("world", Some("alt"))];

    query!(conn, INSERT INTO CustomFunctionTestTable VALUES {test_data}).await?;
    Ok(())
}

// ==============================================
// Custom Function Tests
// ==============================================

/// Custom function name should be case-insensitive in query! macros.
#[always_context(skip(!))]
#[tokio::test]
async fn test_custom_function_case_insensitive() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_custom_test_data(&mut conn).await?;

    let result: CustomFunctionTestData = query!(
        &mut conn,
        SELECT CustomFunctionTestData FROM CustomFunctionTestTable
        WHERE FaNcYuPpEr(name) = "HELLO"
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Custom function with multiple allowed argument counts (2 args).
#[always_context(skip(!))]
#[tokio::test]
async fn test_custom_function_multiple_args_two() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_custom_test_data(&mut conn).await?;

    let result: CustomFunctionTestData = query!(
        &mut conn,
        SELECT CustomFunctionTestData FROM CustomFunctionTestTable
        WHERE slice(name, 2) = "ello"
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Custom function with multiple allowed argument counts (3 args).
#[always_context(skip(!))]
#[tokio::test]
async fn test_custom_function_multiple_args_three() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_custom_test_data(&mut conn).await?;

    let result: CustomFunctionTestData = query!(
        &mut conn,
        SELECT CustomFunctionTestData FROM CustomFunctionTestTable
        WHERE slice(name, 2, 3) = "ell"
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Variadic custom function (Any args) with two and three arguments.
#[always_context(skip(!))]
#[tokio::test]
async fn test_custom_function_any_args() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_custom_test_data(&mut conn).await?;

    let result_two: CustomFunctionTestData = query!(
        &mut conn,
        SELECT CustomFunctionTestData FROM CustomFunctionTestTable
        WHERE anycoalesce(alt_name, name) = "hello"
    )
    .await?;

    let result_three: CustomFunctionTestData = query!(
        &mut conn,
        SELECT CustomFunctionTestData FROM CustomFunctionTestTable
        WHERE AnyCoalesce(alt_name, name, "fallback") = "hello"
    )
    .await?;

    assert_eq!(result_two.name, "hello");
    assert_eq!(result_three.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// The generated macro_rules! should return the SQL function name.
#[test]
fn test_custom_function_macro_rules_output() {
    let sql_name = fancyupper!(1);
    assert_eq!(sql_name, "UPPER");
}
