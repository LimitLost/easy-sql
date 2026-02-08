use super::Database;
use super::TestDriver;
use crate::{Output, Table};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;

#[derive(Table)]
#[sql(no_version)]
struct CustomSelectTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    first_name: String,
    last_name: String,
    age: i64,
}

#[derive(crate::Insert)]
#[sql(table = CustomSelectTable)]
#[sql(default = id)]
struct CustomSelectInsert {
    first_name: String,
    last_name: String,
    age: i64,
}
#[cfg(not(feature = "use_output_columns"))]
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect1 {
    id: i32,
    first_name: String,
    last_name: String,
    #[sql(select = age * 2)]
    agee: i64,
    #[sql(select = CustomSelectTable.age * 4)]
    ageee: i64,
}
#[cfg(feature = "use_output_columns")]
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect1 {
    id: i32,
    first_name: String,
    last_name: String,
    #[sql(select = age * 2)]
    agee: i64,
    #[sql(select = CustomSelectTable.age * 4)]
    ageee: i64,
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

// Test custom select with arg used as parameter (age multiplied by arg0)
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect3 {
    id: i32,
    #[sql(select = age * {arg0})]
    age_times: i64,
}

// Test custom select with multiple args (string concat + arithmetic)
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect4 {
    id: i32,
    #[sql(select = first_name || {arg0} || last_name)]
    full_name: String,
    #[sql(select = age + {arg1})]
    age_plus: i64,
}

// Test custom select with complex SQL expressions in args
#[derive(Output)]
#[sql(table = CustomSelectTable)]
#[allow(unused)]
struct CustomSelect5 {
    id: i32,
    #[sql(select = age + {arg0})]
    adjusted_age: i64,
    #[sql(select = first_name || {arg1})]
    labeled_name: String,
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

    // Query with custom select - should get age * 2
    let result: CustomSelect1 = query!(&mut conn,
        SELECT CustomSelect1 FROM CustomSelectTable WHERE id = 1
    )
    .await?;

    // Verify the custom select expression (age * 2)
    assert_eq!(result.id, 1);
    assert_eq!(result.first_name, "John");
    assert_eq!(result.last_name, "Doe");
    assert_eq!(result.agee, 60, "Expected age * 2 = 30 * 2 = 60");
    assert_eq!(result.ageee, 120, "Expected age * 2 * 2 = 30 * 2 * 2 = 120");

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_output_args_string_concat() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Jane".to_string(),
            last_name: "Smith".to_string(),
            age: 25,
        }
    })
    .await?;

    let result: CustomSelect2 = query!(&mut conn,
        SELECT CustomSelect2(" Jr.") FROM CustomSelectTable WHERE id = 1
    )
    .await?;

    assert_eq!(result.first_name, "Jane");
    assert_eq!(result.last_name, "Smith Jr.");

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_output_args_numeric() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Leo".to_string(),
            last_name: "Ng".to_string(),
            age: 12,
        }
    })
    .await?;

    let result: CustomSelect3 = query!(&mut conn,
        SELECT CustomSelect3(3) FROM CustomSelectTable WHERE id = 1
    )
    .await?;

    assert_eq!(
        result.age_times, 36,
        "Expected age * multiplier = 12 * 3 = 36"
    );

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_output_args_multiple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Ada".to_string(),
            last_name: "Lovelace".to_string(),
            age: 36,
        }
    })
    .await?;

    let result: CustomSelect4 = query!(&mut conn,
        SELECT CustomSelect4(" ", 4) FROM CustomSelectTable WHERE id = 1
    )
    .await?;

    assert_eq!(result.full_name, "Ada Lovelace");
    assert_eq!(result.age_plus, 40);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_output_args_complex_expressions() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Grace".to_string(),
            last_name: "Hopper".to_string(),
            age: 85,
        }
    })
    .await?;

    let result: CustomSelect5 = query!(&mut conn,
        SELECT CustomSelect5(5 + 3 * 2, " #" || CustomSelectTable.last_name) FROM CustomSelectTable WHERE id = 1
    )
    .await?;

    assert_eq!(
        result.adjusted_age, 96,
        "Expected age + (5 + 3 * 2) = 85 + 11 = 96"
    );
    assert_eq!(result.labeled_name, "Grace #Hopper");

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_with_output_args_external_vars() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Nikola".to_string(),
            last_name: "Tesla".to_string(),
            age: 50,
        }
    })
    .await?;

    let separator = " ";
    let age_offset = 7;
    let result: CustomSelect4 = query!(&mut conn,
        SELECT CustomSelect4({separator}, {age_offset}) FROM CustomSelectTable WHERE id = 1
    )
    .await?;

    assert_eq!(result.full_name, "Nikola Tesla");
    assert_eq!(result.age_plus, 57);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_select_output_args_with_clause_vars() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    for (first_name, last_name, age) in [
        ("Ada", "Lovelace", 36),
        ("Grace", "Hopper", 85),
        ("Katherine", "Johnson", 101),
    ] {
        query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
            CustomSelectInsert {
                first_name: first_name.to_string(),
                last_name: last_name.to_string(),
                age,
            }
        })
        .await?;
    }

    let separator = " ";
    let age_offset = 5;
    let min_age = 40;
    let excluded_name = "Ada";
    let min_count = 1;
    let limit_val = 1;

    let results: Vec<CustomSelect4> = query!(&mut conn,
        SELECT Vec<CustomSelect4>({separator}, {age_offset})
        FROM CustomSelectTable
        WHERE CustomSelectTable.age >= {min_age} AND CustomSelectTable.first_name != {excluded_name}
        GROUP BY id
        HAVING COUNT(*) >= {min_count}
        ORDER BY id DESC
        LIMIT {limit_val}
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].full_name, "Katherine Johnson");
    assert_eq!(results[0].age_plus, 106);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_insert_returning_with_output_args() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    let inserted: CustomSelect4 = query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Alan".to_string(),
            last_name: "Turing".to_string(),
            age: 41,
        }
    } RETURNING CustomSelect4(" ", 2))
    .await?;

    assert_eq!(inserted.full_name, "Alan Turing");
    assert_eq!(inserted.age_plus, 43);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_returning_with_output_args() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Grace".to_string(),
            last_name: "Hopper".to_string(),
            age: 85,
        }
    })
    .await?;

    let returned: CustomSelect5 = query!(&mut conn,
        UPDATE CustomSelectTable SET age = 90 WHERE id = 1
        RETURNING CustomSelect5(5 + 3 * 2, " #" || CustomSelectTable.last_name)
    )
    .await?;

    assert_eq!(
        returned.adjusted_age, 101,
        "Expected age + 11 = 90 + 11 = 101"
    );
    assert_eq!(returned.labeled_name, "Grace #Hopper");

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_returning_with_output_args() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Katherine".to_string(),
            last_name: "Johnson".to_string(),
            age: 101,
        }
    })
    .await?;

    let deleted: CustomSelect2 = query!(&mut conn,
        DELETE FROM CustomSelectTable WHERE id = 1
        RETURNING CustomSelect2(" Sr.")
    )
    .await?;

    assert_eq!(deleted.first_name, "Katherine");
    assert_eq!(deleted.last_name, "Johnson Sr.");

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_returning_with_output_args_external_vars() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Ada".to_string(),
            last_name: "Lovelace".to_string(),
            age: 36,
        }
    })
    .await?;

    let bump = 9;
    let prefix = " #";
    let returned: CustomSelect5 = query!(&mut conn,
        UPDATE CustomSelectTable SET age = 40 WHERE id = 1
        RETURNING CustomSelect5({bump} + 1, {prefix} || CustomSelectTable.last_name)
    )
    .await?;

    assert_eq!(
        returned.adjusted_age, 50,
        "Expected age + (bump + 1) = 40 + 10 = 50"
    );
    assert_eq!(returned.labeled_name, "Ada #Lovelace");

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_update_output_args_with_clause_vars() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Margaret".to_string(),
            last_name: "Hamilton".to_string(),
            age: 35,
        }
    })
    .await?;

    let separator = " ";
    let age_offset = 4;
    let target_id = 1;
    let target_name = "Margaret";
    let new_age = 40;

    let returned: CustomSelect4 = query!(&mut conn,
        UPDATE CustomSelectTable SET age = {new_age}
        WHERE id = {target_id} AND first_name = {target_name}
        RETURNING CustomSelect4({separator}, {age_offset})
    )
    .await?;

    assert_eq!(returned.full_name, "Margaret Hamilton");
    assert_eq!(returned.age_plus, 44);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_delete_output_args_with_clause_vars() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CustomSelectTable>().await?;
    let mut conn = db.transaction().await?;

    query!(&mut conn, INSERT INTO CustomSelectTable VALUES {
        CustomSelectInsert {
            first_name: "Dorothy".to_string(),
            last_name: "Vaughan".to_string(),
            age: 50,
        }
    })
    .await?;

    let separator = " ";
    let age_offset = 6;
    let min_age = 45;
    let target_name = "Dorothy";

    let deleted: CustomSelect4 = query!(&mut conn,
        DELETE FROM CustomSelectTable
        WHERE age >= {min_age} AND first_name = {target_name}
        RETURNING CustomSelect4({separator}, {age_offset})
    )
    .await?;

    assert_eq!(deleted.full_name, "Dorothy Vaughan");
    assert_eq!(deleted.age_plus, 56);

    conn.rollback().await?;
    Ok(())
}

#[test]
fn test_custom_select_method_generation() {
    // Test that __easy_sql_select is generated correctly
    let delimiter = "\"";
    let select_str = CustomSelect2::__easy_sql_select::<TestDriver>(delimiter, " Suffix");
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

    let result = CustomSelect2::__easy_sql_select::<TestDriver>("\"", "TestValue");

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
        let select_str = CustomSelect2::__easy_sql_select::<TestDriver>(delimiter, arg);
        assert_eq!(
            select_str, expected,
            "With delimiter='{}' and arg='{}'\nExpected: {}\nActual:   {}",
            delimiter, arg, expected, select_str
        );
    }
}
