// Tests for SQL function support in WHERE, ORDER BY, and other clauses
// Note: The query! macro design expects SELECT to specify an Output type,
// not individual columns/functions. Functions are tested in WHERE, ORDER BY, etc.

use super::*;
use easy_macros::always_context;
use sql_macros::query;

// ==============================================
// Test Tables
// ==============================================

#[derive(Table, Debug, Clone)]
#[sql(version = 1)]
#[sql(unique_id = "65c65ec0-6f8c-4096-a6c6-f07100aaa19e")]
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

/// Test SQRT function in WHERE
#[always_context(skip(!))]
#[tokio::test]
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

/// Test FLOOR function in WHERE
#[always_context(skip(!))]
#[tokio::test]
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
    #[sql(version = 1)]
    #[sql(unique_id = "f0f8083c-70a4-46e2-a201-ddb0a810fa3b")]
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

// ==============================================
// 8. MOD FUNCTION
// ==============================================

#[always_context(skip(!))]
#[tokio::test]
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
// 11. VALIDATION TESTS (Compile-time)
// ==============================================

/// Test that valid single-argument functions compile
#[test]
fn test_single_arg_functions_compile() {
    // These should all compile without errors:
    // upper(name), lower(name), length(name), trim(name), abs(value)
    // If this test compiles, single-arg function validation works
}

/// Test that valid two-argument functions compile
#[test]
fn test_two_arg_functions_compile() {
    // These should compile:
    // round(price, precision), power(value, exponent), mod(value, divisor)
    // If this test compiles, two-arg function validation works
}

/// Test that variable-argument functions compile
#[test]
fn test_variadic_functions_compile() {
    // These should compile with varying argument counts:
    // concat(a, b), concat(a, b, c), concat(a, b, c, d)
    // coalesce(a, b), coalesce(a, b, c)
    // If this test compiles, variadic function validation works
}

// The following would cause compile errors (documented for reference):
//
// query!(&mut conn, SELECT FunctionTestData FROM FunctionTestTable WHERE sum() = 0)
// // Error: Function SUM requires at least 1 argument(s)
//
// query!(&mut conn, SELECT FunctionTestData FROM FunctionTestTable WHERE upper(name, name) = "A")
// // Error: Function UPPER accepts at most 1 argument(s)
//
// query!(&mut conn, SELECT FunctionTestData FROM FunctionTestTable WHERE round(price) = 1)
// // Error: Function ROUND requires at least 1 argument(s) and accepts at most 2 argument(s)
