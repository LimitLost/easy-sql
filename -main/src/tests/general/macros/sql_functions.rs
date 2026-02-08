// Tests for SQL function support in WHERE, ORDER BY, and other clauses
// Note: The query! macro design expects SELECT to specify an Output type,
// not individual columns/functions. Functions are tested in WHERE, ORDER BY, etc.

use super::*;
use easy_macros::always_context;
use easy_sql_macros::query;

// ==============================================
// Test Tables
// ==============================================

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
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

#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = FunctionTestTable)]
struct FunctionAggregateResults {
    #[sql(select = COUNT(*))]
    count_all: i64,
    #[sql(select = SUM(value))]
    sum_value: i64,
    #[sql(select = cast(AVG(value) AS f64))]
    avg_value: f64,
    #[sql(select = MIN(value))]
    min_value: i32,
    #[sql(select = MAX(value))]
    max_value: i32,
}
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
        test_data("  Spaces  ", 100, 5.0, "Test"),
    ];

    query!(conn, INSERT INTO FunctionTestTable VALUES {test_data}).await?;
    Ok(())
}

// ==============================================
// 1. FUNCTIONS IN WHERE CLAUSES
// ==============================================

/// Test LENGTH function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_length_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Find items where LENGTH(name) > 5
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE length(name) > 5
    )
    .await?;

    // "Banana" (6), "Carrot" (6), "Eggplant" (8), "  Spaces  " (10)
    assert!(
        results.len() >= 3,
        "Should find at least 3 items with name length > 5"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test UPPER function in WHERE for case-insensitive search
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

/// Test LOWER function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_lower_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let search = "banana".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE lower(name) = {search}
    )
    .await?;

    assert_eq!(result.name, "Banana");

    conn.rollback().await?;
    Ok(())
}

/// Test TRIM function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_trim_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let search = "Spaces".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE trim(name) = {search}
    )
    .await?;

    assert_eq!(result.value, 100);

    conn.rollback().await?;
    Ok(())
}

/// Test ABS function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_abs_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert negative value
    let data = test_data("Negative", -42, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let abs_val = 42;
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE abs(value) = {abs_val}
    )
    .await?;

    assert_eq!(result.name, "Negative");
    assert_eq!(result.value, -42);

    conn.rollback().await?;
    Ok(())
}

/// Test ROUND function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_round_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let min_price = 2.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE round(price) >= {min_price}
    )
    .await?;

    // Apple: round(1.50)=2, Dates: round(3.00)=3, Eggplant: round(2.25)=2, Spaces: round(5.0)=5
    assert_eq!(
        results.len(),
        4,
        "Should find 4 items with rounded price >= 2"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test ROUND with precision argument (SQLite)
#[always_context(skip(!))]
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_function_round_with_precision_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let precision = 1;
    let rounded = 1.5;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE round(cast(price as f64), {precision}) = {rounded}
    )
    .await?;

    assert!(
        results.iter().any(|row| row.name == "Apple"),
        "Should find Apple where round(price, 1) == 1.5"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test ROUND with precision argument (Postgres)
#[always_context(skip(!))]
#[tokio::test]
#[cfg(all(feature = "postgres", feature = "rust_decimal"))]
async fn test_function_round_with_precision_in_where() -> anyhow::Result<()> {
    use rust_decimal::Decimal;

    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let precision = 1;
    let rounded = 1.5;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE round(cast(price as Decimal), {precision}) = {rounded}
    )
    .await?;

    assert!(
        results.iter().any(|row| row.name == "Apple"),
        "Should find Apple where round(price, 1) == 1.5"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 2. NESTED FUNCTIONS IN WHERE
// ==============================================

/// Test nested string functions in WHERE
#[always_context(skip(!))]
#[tokio::test]
async fn test_nested_functions_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let search = "SPACES".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE upper(trim(name)) = {search}
    )
    .await?;

    assert_eq!(result.value, 100);

    conn.rollback().await?;
    Ok(())
}

/// Test nested math functions in WHERE  
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_nested_math_functions_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = test_data("MathTest", -25, 1.0, "Test");
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    // SQRT(ABS(value)) = SQRT(25) = 5
    let expected = 5.0;
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE sqrt(abs(value)) = {expected}
    )
    .await?;

    assert_eq!(result.name, "MathTest");
    assert_eq!(result.value, -25);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3. FUNCTIONS IN ORDER BY
// ==============================================

/// Test LENGTH function in ORDER BY clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_in_order_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE category != "Test"
        ORDER BY length(name)
    )
    .await?;

    // Shortest to longest (excluding "Test"): Apple(5), Dates(5), Banana(6), Carrot(6), Eggplant(8)
    assert!(results.len() >= 5);
    // Last item should have longest name
    assert_eq!(results.last().unwrap().name, "Eggplant");

    conn.rollback().await?;
    Ok(())
}

/// Test ABS function in ORDER BY
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_abs_in_order_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = vec![
        test_data("A", -30, 1.0, "Test"),
        test_data("B", -10, 1.0, "Test"),
        test_data("C", 20, 1.0, "Test"),
        test_data("D", -25, 1.0, "Test"),
    ];
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE category = "Test"
        ORDER BY abs(value)
    )
    .await?;

    // Ordered by absolute value: B(10), C(20), D(25), A(30)
    assert_eq!(results[0].name, "B"); // abs(-10) = 10
    assert_eq!(results[1].name, "C"); // abs(20) = 20
    assert_eq!(results[2].name, "D"); // abs(-25) = 25
    assert_eq!(results[3].name, "A"); // abs(-30) = 30

    conn.rollback().await?;
    Ok(())
}

/// Test aggregate functions in SELECT
#[always_context(skip(!))]
#[tokio::test]
async fn test_aggregate_functions_in_select() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let result: FunctionAggregateResults = query!(&mut conn,
        SELECT FunctionAggregateResults FROM FunctionTestTable
    )
    .await?;

    assert!(result.count_all >= 6);
    assert!(result.sum_value > 0);
    assert!(result.avg_value > 0.0);
    assert!(result.min_value <= result.max_value);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 4. COMPLEX EXPRESSIONS WITH FUNCTIONS
// ==============================================

/// Test function in arithmetic expression in WHERE
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_in_arithmetic_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // WHERE value * ROUND(price) > 15
    let threshold = 15.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE value * round(price) > {threshold}
    )
    .await?;

    // Apple: 10 * 2 = 20 > 15 ✓
    // Banana: 20 * 1 = 20 > 15 ✓
    // Others: should be less
    assert!(
        results.len() >= 2,
        "Should find items where value * round(price) > 15"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test multiple functions in same WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_multiple_functions_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let min_length = 6;
    let min_price = 2.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE length(name) >= {min_length} and round(price) >= {min_price}
    )
    .await?;

    // Must have name length >= 6 AND rounded price >= 2
    // Eggplant: 8 chars, price rounds to 2 ✓
    assert!(
        !results.is_empty(),
        "Should find at least one item matching both conditions"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test SUBSTRING function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_substring_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let start = 1;
    let length = 3;
    let expected = "Ban".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE substring(name, {start}, {length}) = {expected}
    )
    .await?;

    assert_eq!(result.name, "Banana");

    conn.rollback().await?;
    Ok(())
}

/// Test SUBSTR function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_substr_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let start = 1;
    let length = 5;
    let expected = "Apple".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE substr(name, {start}, {length}) = {expected}
    )
    .await?;

    assert_eq!(result.name, "Apple");

    conn.rollback().await?;
    Ok(())
}

/// Test NULLIF function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_nullif_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let target = 20;
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE nullif(value, {target}) IS NULL
    )
    .await?;

    assert_eq!(result.value, 20);

    conn.rollback().await?;
    Ok(())
}

/// Test function with OR in WHERE
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_with_or_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE length(name) > 7 or round(price) = 1
    )
    .await?;

    // Either name length > 7 OR rounded price = 1
    // "Eggplant" has 8 chars, "Banana" rounds to 1, "Apple" rounds to 2
    assert!(
        results.len() >= 2,
        "Should find items matching either condition"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 5. POWER AND SQRT FUNCTIONS (PostgreSQL only)
// ==============================================

/// Test POWER function in WHERE
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_power_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = vec![
        test_data("Two", 2, 1.0, "Math"),
        test_data("Three", 3, 1.0, "Math"),
        test_data("Four", 4, 1.0, "Math"),
    ];
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    // Find where value^2 >= 9 (3^2=9, 4^2=16)
    let exponent = 2;
    let threshold = 9.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE power(value, {exponent}) >= {threshold}
    )
    .await?;

    assert_eq!(results.len(), 2, "Should find 2 items (3 and 4)");

    conn.rollback().await?;
    Ok(())
}

/// Test POW function in WHERE
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_pow_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = vec![
        test_data("Two", 2, 1.0, "Math"),
        test_data("Three", 3, 1.0, "Math"),
    ];
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    let exponent = 2;
    let threshold = 4.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE pow(value, {exponent}) >= {threshold}
    )
    .await?;

    assert_eq!(results.len(), 2, "Should find 2 items (2 and 3)");

    conn.rollback().await?;
    Ok(())
}

/// Test SQRT function in WHERE
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_sqrt_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = vec![
        test_data("Sq4", 4, 1.0, "Math"),
        test_data("Sq9", 9, 1.0, "Math"),
        test_data("Sq16", 16, 1.0, "Math"),
    ];
    query!(&mut conn, INSERT INTO FunctionTestTable VALUES {data}).await?;

    // Find where sqrt(value) >= 3 (sqrt(9)=3, sqrt(16)=4)
    let threshold = 3.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE sqrt(value) >= {threshold}
    )
    .await?;

    assert_eq!(results.len(), 2, "Should find 2 items (9 and 16)");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 6. CEIL AND FLOOR FUNCTIONS
// ==============================================

/// Test CEIL function in WHERE
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_ceil_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Find items where ceil(price) >= 2
    let threshold = 2.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE ceil(price) >= {threshold}
    )
    .await?;

    // Apple: ceil(1.50)=2, Dates: ceil(3.00)=3, Eggplant: ceil(2.25)=3
    assert!(
        results.len() >= 3,
        "Should find items with ceil(price) >= 2"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test CEILING function in WHERE
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_ceiling_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let threshold = 3.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE ceiling(price) >= {threshold}
    )
    .await?;

    assert!(
        results.iter().any(|row| row.name == "Dates"),
        "Should find Dates where ceiling(price) >= 3"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test FLOOR function in WHERE
#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_floor_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Find items where floor(price) = 0
    let floor_val = 0.0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE floor(price) = {floor_val}
    )
    .await?;

    // Banana: floor(0.75)=0, Carrot: floor(0.50)=0
    assert_eq!(
        results.len(),
        2,
        "Should find 2 items with floor(price) = 0"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 7. COALESCE FUNCTION
// ==============================================

/// Test COALESCE function with nullable fields
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_coalesce() -> anyhow::Result<()> {
    // Create table with nullable column
    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    #[sql(unique_id = "22eaea17-209b-40b8-9a96-bcbbc7ba54fa")]
    struct NullableTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        nullable_value: Option<i32>,
        default_value: i32,
    }

    #[derive(Insert, Output, Debug, Clone)]
    #[sql(table = NullableTable)]
    #[sql(default = id)]
    struct NullableData {
        nullable_value: Option<i32>,
        default_value: i32,
    }

    let db = Database::setup_for_testing::<NullableTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert rows with NULL and non-NULL
    let data = vec![
        NullableData {
            nullable_value: None,
            default_value: 42,
        },
        NullableData {
            nullable_value: Some(100),
            default_value: 42,
        },
    ];
    query!(&mut conn, INSERT INTO NullableTable VALUES {data}).await?;

    // COALESCE should use default when nullable is NULL
    let default_val = 42;
    let result: NullableData = query!(&mut conn,
        SELECT NullableData FROM NullableTable
        WHERE coalesce(nullable_value, default_value) = {default_val}
    )
    .await?;

    // Should find the row where nullable is NULL (coalesces to 42)
    assert_eq!(result.nullable_value, None);
    assert_eq!(result.default_value, 42);

    conn.rollback().await?;
    Ok(())
}

/// Test IFNULL function with nullable fields (SQLite only)
#[always_context(skip(!))]
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_function_ifnull() -> anyhow::Result<()> {
    #[derive(Table, Debug, Clone)]
    #[sql(no_version)]
    #[sql(unique_id = "1e52a082-c9f9-4b8e-98a5-43a5b5c1f1a8")]
    struct IfNullTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        nullable_value: Option<i32>,
        default_value: i32,
    }

    #[derive(Insert, Output, Debug, Clone)]
    #[sql(table = IfNullTable)]
    #[sql(default = id)]
    struct IfNullData {
        nullable_value: Option<i32>,
        default_value: i32,
    }

    let db = Database::setup_for_testing::<IfNullTable>().await?;
    let mut conn = db.transaction().await?;

    let data = vec![
        IfNullData {
            nullable_value: None,
            default_value: 7,
        },
        IfNullData {
            nullable_value: Some(5),
            default_value: 7,
        },
    ];
    query!(&mut conn, INSERT INTO IfNullTable VALUES {data}).await?;

    let result: IfNullData = query!(&mut conn,
        SELECT IfNullData FROM IfNullTable
        WHERE ifnull(nullable_value, default_value) = 7
    )
    .await?;

    assert_eq!(result.nullable_value, None);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 8. MOD FUNCTION
// ==============================================

#[always_context(skip(!))]
#[tokio::test]
#[cfg(any(feature = "sqlite_math", not(feature = "sqlite")))]
async fn test_function_mod_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Find items where value MOD 5 = 0 (multiples of 5)
    let divisor = 5;
    let remainder = 0;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE mod(value, {divisor}) = {remainder}
    )
    .await?;

    // Should find: Apple(10), Banana(20), Carrot(15), Dates(5), Spaces(100)
    assert!(results.len() >= 5, "Should find multiples of 5");

    conn.rollback().await?;
    Ok(())
}

/// Test CAST function in WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_cast_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let value_str = "10".to_string();
    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE cast(value as String) = {value_str}
    )
    .await?;

    assert_eq!(result.name, "Apple");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 9. FUNCTION WITH LIMIT
// ==============================================

/// Test function in WHERE with LIMIT
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_with_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let min_length = 5;
    let limit = 3;
    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE length(name) >= {min_length}
        ORDER BY name
        LIMIT {limit}
    )
    .await?;

    assert_eq!(
        results.len(),
        3,
        "Should return exactly 3 results due to LIMIT"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 10. CONCAT FUNCTION
// ==============================================

/// Test CONCAT function in WHERE (for databases that support it)
#[always_context(skip(!))]
#[tokio::test]
async fn test_function_concat_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    // Search for concatenated value
    let separator = " - ".to_string();
    let search = "Apple - Fruit".to_string();

    let result: FunctionTestData = query!(&mut conn,
        SELECT FunctionTestData FROM FunctionTestTable
        WHERE concat(name, {separator}, category) = {search}
    )
    .await?;

    assert_eq!(result.name, "Apple");
    assert_eq!(result.category, "Fruit");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 11. DATE/TIME FUNCTIONS
// ==============================================

/// Test CURRENT_* and DATE/TIME functions (Postgres)
#[always_context(skip(!))]
#[tokio::test]
#[cfg(not(feature = "sqlite"))]
async fn test_function_current_date_time_postgres() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE current_timestamp >= current_timestamp
            AND current_date = current_date
            AND current_time = current_time
    )
    .await?;

    assert!(!results.is_empty());

    conn.rollback().await?;
    Ok(())
}

/// Test NOW function (Postgres)
#[always_context(skip(!))]
#[tokio::test]
#[cfg(not(feature = "sqlite"))]
async fn test_function_now_in_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE now() <= now()
    )
    .await?;

    assert!(!results.is_empty());

    conn.rollback().await?;
    Ok(())
}

/// Test CURRENT_* and DATE/TIME/DATETIME functions (SQLite)
#[always_context(skip(!))]
#[tokio::test]
#[cfg(feature = "sqlite")]
async fn test_function_current_date_time_sqlite() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<FunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    setup_test_data(&mut conn).await?;

    let results: Vec<FunctionTestData> = query!(&mut conn,
        SELECT Vec<FunctionTestData> FROM FunctionTestTable
        WHERE current_timestamp >= current_timestamp
            AND current_date = current_date
            AND current_time = current_time
            AND date(current_timestamp) = date(current_timestamp)
            AND time(current_timestamp) = time(current_timestamp)
            AND datetime(current_timestamp) = datetime(current_timestamp)
    )
    .await?;

    assert!(!results.is_empty());

    conn.rollback().await?;
    Ok(())
}
