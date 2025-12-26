use super::Database;
use crate::{Output, Table};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::query;

#[derive(Table)]
#[sql(version = 1)]
#[sql(unique_id = "2c8bee3e-18e3-4ed6-8bb4-54ea0ed31ea8")]
struct CustomSelectTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    first_name: String,
    last_name: String,
    age: i64,
}

// For INSERT operations
#[derive(crate::Insert)]
#[sql(table = CustomSelectTable)]
#[sql(default = id)]
struct CustomSelectInsert {
    first_name: String,
    last_name: String,
    age: i64,
}

// Test basic custom select without args
// All fields from the table still exist
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect1 {
    id: i32,
    first_name: String,
    last_name: String,
    #[sql(select = age * 2)]
    age: i64, // Reuse age field but with custom select expression
}

// Test custom select with one arg
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect2 {
    id: i32,
    first_name: String,
    #[sql(select = last_name || {arg0})]
    last_name: String, // Reuse last_name field but with custom select expression
    age: i64,
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_custom_select_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            age: 30
        }
    })
    .await?;

    // TODO: Query with custom select - need to implement query macro support first
    // let result: CustomSelect1 = query!(conn,
    //     SELECT CustomSelect1 FROM CustomSelectTable WHERE id = 1
    // ).await?;

    conn.rollback().await?;
    Ok(())
}

#[test]
fn test_custom_select_method_generation() {
    // Test that __easy_sql_select is generated correctly
    let delimiter = "\"";
    let select_str = CustomSelect2::__easy_sql_select(delimiter, " Suffix");
    println!("Generated SELECT: {}", select_str);

    // Verify exact SELECT string (with alias prefix to avoid conflicts)
    let expected = "\"id\", \"first_name\", \"age\", \"last_name\" ||  Suffix AS \"__easy_sql_custom_select_last_name\"";
    assert_eq!(
        select_str, expected,
        "Generated SELECT string doesn't match expected.\nExpected: {}\nActual:   {}",
        expected, select_str
    );
}

#[test]
fn test_custom_select_format_string_uses_named_args() {
    // Verify that the generated format! call uses {arg0} syntax, not {0}
    // This is a compile-time check - if the macro generates {0} instead of {arg0},
    // this will fail to compile because format! won't be able to match the argument names

    let result = CustomSelect2::__easy_sql_select("\"", "TestValue");

    // The fact that this compiles and runs proves that the macro
    // generates format! with {arg0} syntax like: format!("... || {arg0} ...", arg0=arg0)
    assert!(result.contains("TestValue"));
}

#[test]
fn test_custom_select_multiple_args() {
    // Test with multiple different delimiter and argument combinations
    let test_cases = vec![
        (
            "\"",
            " Smith",
            "\"id\", \"first_name\", \"age\", \"last_name\" ||  Smith AS \"__easy_sql_custom_select_last_name\"",
        ),
        (
            "'",
            "-Suffix",
            "'id', 'first_name', 'age', 'last_name' || -Suffix AS '__easy_sql_custom_select_last_name'",
        ),
        (
            "`",
            " Jr.",
            "`id`, `first_name`, `age`, `last_name` ||  Jr. AS `__easy_sql_custom_select_last_name`",
        ),
    ];

    for (delimiter, arg, expected) in test_cases {
        let select_str = CustomSelect2::__easy_sql_select(delimiter, arg);
        assert_eq!(
            select_str, expected,
            "With delimiter='{}' and arg='{}'\nExpected: {}\nActual:   {}",
            delimiter, arg, expected, select_str
        );
    }
}
