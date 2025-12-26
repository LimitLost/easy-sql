// Comprehensive tests for SQL function support (both built-in and custom functions)
// Tests function calls in SELECT, WHERE, and other clauses

use super::*;
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::query;

// ==============================================
// Test Tables for Function Testing
// ==============================================

/// Table for testing aggregate functions
#[derive(Table, Debug, Clone)]
#[sql(version = 1)]
#[sql(unique_id = "595b7d92-bd05-4327-840e-49f07d581588")]
pub struct FunctionTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub name: String,
    pub value: i32,
    pub price: f64,
    pub category: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = FunctionTestTable)]
#[sql(default = id)]
pub struct FunctionTestData {
    pub name: String,
    pub value: i32,
    pub price: f64,
    pub category: String,
}

// Note: For function results, we typically use primitive types directly
// rather than Output structs, since function results don't correspond
// to table fields. Output structs are only needed when selecting
// multiple values that need to be grouped together.

// ==============================================
// Test Utilities
// ==============================================

fn test_data(name: &str, value: i32, price: f64, category: &str) -> FunctionTestData {
    FunctionTestData {
        name: name.to_string(),
        value,
        price,
        category: category.to_string(),
    }
}

#[always_context(skip(!))]
async fn setup_test_data(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
) -> anyhow::Result<()> {
    let test_data = vec![
        test_data("Apple", 10, 1.50, "Fruit"),
        test_data("Banana", 20, 0.75, "Fruit"),
        test_data("Carrot", 15, 0.50, "Vegetable"),
        test_data("Dates", 5, 3.00, "Fruit"),
        test_data("Eggplant", 8, 2.25, "Vegetable"),
    ];

    query!(conn, INSERT INTO FunctionTestTable VALUES {test_data}).await?;
    Ok(())
}

// ==============================================
// 1. AGGREGATE FUNCTIONS
// ==============================================

/// Test COUNT function - basic count
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_count_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Count all rows
    let result: i64 = query!(&mut conn,
        SELECT count(id) FROM FunctionTestTable WHERE true
    )
    .await?;

    assert_eq!(result, 5, "COUNT should return 5 rows");

    conn.rollback().await?;
    Ok(())
}

/// Test COUNT with WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_count_with_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let category = "Fruit".to_string();
    let result: i64 = query!(&mut conn,
        SELECT count(id) FROM FunctionTestTable WHERE category = {category}
    )
    .await?;

    assert_eq!(result, 3, "COUNT should return 3 fruits");

    conn.rollback().await?;
    Ok(())
}

/// Test SUM function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_sum() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: i64 = query!(&mut conn,
        SELECT sum(value) FROM FunctionTestTable WHERE true
    )
    .await?;

    assert_eq!(result, 58, "SUM should return 10+20+15+5+8 = 58");

    conn.rollback().await?;
    Ok(())
}

/// Test AVG function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_avg() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: f64 = query!(&mut conn,
        SELECT avg(value) FROM FunctionTestTable WHERE true
    )
    .await?;

    // Average of 10, 20, 15, 5, 8 = 58/5 = 11.6
    assert!(
        (result - 11.6).abs() < 0.1,
        "AVG should be approximately 11.6"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test MIN function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_min() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: i32 = query!(&mut conn,
        SELECT min(value) FROM FunctionTestTable WHERE true
    )
    .await?;

    assert_eq!(result, 5, "MIN should return 5");

    conn.rollback().await?;
    Ok(())
}

/// Test MAX function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_max() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: i32 = query!(&mut conn,
        SELECT max(value) FROM FunctionTestTable WHERE true
    )
    .await?;

    assert_eq!(result, 20, "MAX should return 20");

    conn.rollback().await?;
    Ok(())
}

/// Test multiple aggregate functions in one query
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_multiple_aggregates() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // For aggregates that return multiple values, we define a struct
    // inline without Output derive, since function results don't map to table fields
    #[derive(Debug, sqlx::FromRow)]
    struct MultiAggResult {
        count_val: i64,
        sum_val: i64,
        avg_val: f64,
        min_val: i32,
        max_val: i32,
    }

    let result: MultiAggResult = query!(&mut conn,
        SELECT count(id), sum(value), avg(value), min(value), max(value)
        FROM FunctionTestTable WHERE true
    )
    .await?;

    assert_eq!(result.count_val, 5);
    assert_eq!(result.sum_val, 58);
    assert!((result.avg_val - 11.6).abs() < 0.1);
    assert_eq!(result.min_val, 5);
    assert_eq!(result.max_val, 20);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 2. STRING FUNCTIONS
// ==============================================

/// Test UPPER function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_upper() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: String = query!(&mut conn,
        SELECT upper(name) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result, "APPLE", "UPPER should convert to uppercase");

    conn.rollback().await?;
    Ok(())
}

/// Test LOWER function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_lower() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: String = query!(&mut conn,
        SELECT lower(name) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result, "apple", "LOWER should convert to lowercase");

    conn.rollback().await?;
    Ok(())
}

/// Test LENGTH function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_length() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: i32 = query!(&mut conn,
        SELECT length(name) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result, 5, "LENGTH of 'Apple' should be 5");

    conn.rollback().await?;
    Ok(())
}

/// Test CONCAT function with multiple arguments
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_concat() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let separator = " - ".to_string();
    let result: String = query!(&mut conn,
        SELECT concat(name, {separator}, category) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result, "Apple - Fruit", "CONCAT should concatenate strings");

    conn.rollback().await?;
    Ok(())
}

/// Test TRIM function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_trim() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert data with spaces
    let data = test_data("  Spaces  ", 100, 5.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let result: String = query!(&mut conn,
        SELECT trim(name) FROM FunctionTestTable WHERE value = 100
    )
    .await?;

    assert_eq!(
        result, "Spaces",
        "TRIM should remove leading/trailing spaces"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3. MATH FUNCTIONS
// ==============================================

/// Test ABS function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_abs() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert negative value
    let data = test_data("Negative", -42, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let result: i32 = query!(&mut conn,
        SELECT abs(value) FROM FunctionTestTable WHERE value = -42
    )
    .await?;

    assert_eq!(result, 42, "ABS should return absolute value");

    conn.rollback().await?;
    Ok(())
}

/// Test ROUND function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_round() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: f64 = query!(&mut conn,
        SELECT round(price, 1) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert!(
        (result - 1.5).abs() < 0.01,
        "ROUND should round to 1 decimal place"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test CEIL/CEILING function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_ceil() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: f64 = query!(&mut conn,
        SELECT ceil(price) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result, 2.0, "CEIL should round up to 2.0");

    conn.rollback().await?;
    Ok(())
}

/// Test FLOOR function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_floor() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: f64 = query!(&mut conn,
        SELECT floor(price) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result, 1.0, "FLOOR should round down to 1.0");

    conn.rollback().await?;
    Ok(())
}

/// Test POWER function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_power() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = test_data("PowerTest", 2, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let exponent = 3;
    let result: f64 = query!(&mut conn,
        SELECT power(value, {exponent}) FROM FunctionTestTable WHERE value = 2
    )
    .await?;

    assert_eq!(result, 8.0, "POWER(2, 3) should return 8.0");

    conn.rollback().await?;
    Ok(())
}

/// Test SQRT function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_sqrt() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = test_data("SqrtTest", 16, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let result: f64 = query!(&mut conn,
        SELECT sqrt(value) FROM FunctionTestTable WHERE value = 16
    )
    .await?;

    assert_eq!(result, 4.0, "SQRT(16) should return 4.0");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 4. NESTED FUNCTIONS
// ==============================================

/// Test nested string functions
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_nested_strings() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // UPPER(TRIM(name))
    let data = test_data("  Nested  ", 100, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let result: String = query!(&mut conn,
        SELECT upper(trim(name)) FROM FunctionTestTable WHERE value = 100
    )
    .await?;

    assert_eq!(result, "NESTED", "Nested functions should work");

    conn.rollback().await?;
    Ok(())
}

/// Test nested math functions
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_nested_math() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = test_data("MathTest", -5, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    // SQRT(ABS(value))
    let result: f64 = query!(&mut conn,
        SELECT sqrt(abs(value)) FROM FunctionTestTable WHERE value = -5
    )
    .await?;

    assert!(
        (result - 2.236).abs() < 0.01,
        "SQRT(ABS(-5)) should be ~2.236"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test function with aggregate
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_with_aggregate() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // ROUND(AVG(price), 2)
    let result: f64 = query!(&mut conn,
        SELECT round(avg(price), 2) FROM FunctionTestTable WHERE true
    )
    .await?;

    // Average: (1.50 + 0.75 + 0.50 + 3.00 + 2.25) / 5 = 8.0 / 5 = 1.6
    assert!(
        (result - 1.6).abs() < 0.01,
        "ROUND(AVG(price), 2) should be 1.60"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 5. FUNCTIONS IN WHERE CLAUSES
// ==============================================

/// Test function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_in_where_clause() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Find items where LENGTH(name) > 5
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE length(name) > 5
    )
    .await?;

    // "Banana" (6), "Carrot" (6), "Eggplant" (8)
    assert_eq!(results.len(), 3, "Should find 3 items with name length > 5");

    conn.rollback().await?;
    Ok(())
}

/// Test UPPER in WHERE for case-insensitive search
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_upper_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let search = "APPLE".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE upper(name) = {search}
    )
    .await?;

    assert_eq!(result.name, "Apple");

    conn.rollback().await?;
    Ok(())
}

/// Test math function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_math_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Find items where ROUND(price) >= 2
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE round(price) >= 2
    )
    .await?;

    // "Dates" (3.00) and "Eggplant" (2.25)
    assert_eq!(
        results.len(),
        2,
        "Should find 2 items with rounded price >= 2"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 6. FUNCTIONS IN ORDER BY
// ==============================================

/// Test function in ORDER BY clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_in_order_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Order by LENGTH(name)
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE true
        ORDER BY length(name)
    )
    .await?;

    // Shortest to longest: Apple(5), Dates(5), Banana(6), Carrot(6), Eggplant(8)
    assert_eq!(results.len(), 5);
    assert_eq!(results[4].name, "Eggplant", "Longest name should be last");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 7. CONDITIONAL FUNCTIONS
// ==============================================

/// Test COALESCE function
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_coalesce() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Create table with nullable column for proper COALESCE test
    #[derive(Table, Debug, Clone)]
    #[sql(version = 1)]
    #[sql(unique_id = "091ed233-63ca-46dc-aaa8-438a51ccbcc2")]
    struct NullableTestTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        nullable_value: Option<i32>,
        default_value: i32,
    }

    #[derive(Insert, Output, Debug, Clone)]
    #[sql(table = NullableTestTable)]
    #[sql(default = id)]
    struct NullableTestData {
        nullable_value: Option<i32>,
        default_value: i32,
    }

    let db2 = Database::setup_for_testing::<NullableTestTable>().await?;
    let mut conn2 = db2.transaction().await?;

    // Insert row with NULL
    let data = NullableTestData {
        nullable_value: None,
        default_value: 42,
    };
    query!(&mut conn2, INSERT INTO NullableTestTable VALUES {data}).await?;

    // COALESCE should return the default value when nullable is NULL
    let result: i32 = query!(&mut conn2,
        SELECT coalesce(nullable_value, default_value) FROM NullableTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(
        result, 42,
        "COALESCE should return default_value when nullable is NULL"
    );

    conn2.rollback().await?;
    Ok(())
}

// ==============================================
// 8. FUNCTIONS WITH PARAMETERS
// ==============================================

/// Test function with Rust variable as argument
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_with_variable_arg() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let min_length = 6;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE length(name) >= {min_length}
    )
    .await?;

    assert_eq!(results.len(), 3, "Should find 3 items with length >= 6");

    conn.rollback().await?;
    Ok(())
}

/// Test CONCAT with multiple variables
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_concat_with_variables() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let prefix = "Product: ".to_string();
    let suffix = " [Fresh]".to_string();

    let result: String = query!(&mut conn,
        SELECT concat({prefix}, name, {suffix}) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(
        result, "Product: Apple [Fresh]",
        "CONCAT with variables should work"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 9. COMPLEX EXPRESSIONS WITH FUNCTIONS
// ==============================================

/// Test function in complex arithmetic expression
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_in_arithmetic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Calculate: value * ROUND(price)
    #[derive(Output, Debug)]
    #[sql(table = FunctionTestTable)]
    struct CalcResult {
        calc: f64,
    }

    let result: CalcResult = query!(&mut conn,
        SELECT value * round(price) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    // value=10, price=1.50, round(1.50)=2, 10*2=20
    assert_eq!(result.calc, 20.0, "Arithmetic with function should work");

    conn.rollback().await?;
    Ok(())
}

/// Test multiple functions in SELECT with calculations
#[always_context(skip(!))]
#[tokio::test]
async fn test_complex_select_with_functions() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    #[derive(Output, Debug)]
    #[sql(table = FunctionTestTable)]
    struct ComplexResult {
        upper_name: String,
        name_length: i32,
        rounded_price: f64,
        abs_value: i32,
    }

    let result: ComplexResult = query!(&mut conn,
        SELECT upper(name), length(name), round(price, 1), abs(value)
        FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    assert_eq!(result.upper_name, "APPLE");
    assert_eq!(result.name_length, 5);
    assert!((result.rounded_price - 1.5).abs() < 0.01);
    assert_eq!(result.abs_value, 10);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 10. ERROR CASES (Compile-time validation)
// ==============================================

// Note: These tests verify compile-time validation by ensuring they compile.
// Actual error cases would fail at compile time, not runtime.

/// Test that single-argument functions work correctly
#[always_context(skip(!))]
#[tokio::test]
async fn test_single_arg_function_valid() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // These should all compile (single arg functions)
    let _: String =
        query!(&mut conn, SELECT upper(name) FROM FunctionTestTable WHERE id = 1).await?;
    let _: String =
        query!(&mut conn, SELECT lower(name) FROM FunctionTestTable WHERE id = 1).await?;
    let _: i32 = query!(&mut conn, SELECT length(name) FROM FunctionTestTable WHERE id = 1).await?;
    let _: String =
        query!(&mut conn, SELECT trim(name) FROM FunctionTestTable WHERE id = 1).await?;
    let _: i32 = query!(&mut conn, SELECT abs(value) FROM FunctionTestTable WHERE id = 1).await?;

    conn.rollback().await?;
    Ok(())
}

/// Test that variable-argument functions work correctly
#[always_context(skip(!))]
#[tokio::test]
async fn test_variable_arg_function_valid() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // CONCAT with 2 args
    let sep = "-".to_string();
    let _: String = query!(&mut conn,
        SELECT concat(name, {sep}) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    // CONCAT with 3 args
    let _: String = query!(&mut conn,
        SELECT concat(name, {sep}, category) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    // COALESCE with 2 args
    let default = "default".to_string();
    let _: String = query!(&mut conn,
        SELECT coalesce(name, {default}) FROM FunctionTestTable WHERE id = 1
    )
    .await?;

    conn.rollback().await?;
    Ok(())
}

// Compile-time validation tests (these would fail to compile if validation isn't working):
//
// This should FAIL to compile (too few args):
// query!(&mut conn, SELECT sum() FROM FunctionTestTable WHERE true)
//
// This should FAIL to compile (too many args):
// query!(&mut conn, SELECT upper(name, name) FROM FunctionTestTable WHERE true)
//
// This should FAIL to compile (wrong arg count for ROUND - needs 2):
// query!(&mut conn, SELECT round(price) FROM FunctionTestTable WHERE true)
