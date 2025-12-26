// Test for using Output type columns in custom select expressions

use crate::tests::general::Database;
use crate::{Insert, Output, Table, Update};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::query;

#[derive(Table)]
#[sql(version = 1)]
#[sql(unique_id = "ec51bd8a-2f9b-472d-bac7-0953c8217f3b")]
struct OutputColumnsTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    first_name: String,
    last_name: String,
    age: i32,
}

#[derive(Insert, Update)]
#[sql(table = OutputColumnsTestTable)]
#[sql(default = id)]
struct OutputColumnsTestInsert {
    first_name: String,
    last_name: String,
    age: i32,
}

// Output type with custom select that uses columns from the Output type itself
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = OutputColumnsTestTable)]
#[sql(default = id)]
struct OutputColumnsTestOutput {
    first_name: String,
    last_name: String,
    #[sql(select = OutputColumnsTestOutput.first_name || " " || OutputColumnsTestOutput.last_name)]
    full_name: String,
    #[sql(select = OutputColumnsTestTable.age * 12)]
    age_in_months: i32,
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_output_columns_in_custom_select() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<OutputColumnsTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    query!(&mut conn, INSERT INTO OutputColumnsTestTable VALUES {
        OutputColumnsTestInsert {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            age: 25,
        }
    })
    .await?;

    // Select with custom columns - use query! to see actual SQL in error
    eprintln!("About to execute SELECT query");
    let result: OutputColumnsTestOutput = query!(&mut conn,
        SELECT OutputColumnsTestOutput FROM OutputColumnsTestTable WHERE OutputColumnsTestTable.id = 1
    )
    .await?;

    assert_eq!(result.first_name, "John");
    assert_eq!(result.last_name, "Doe");
    assert_eq!(result.full_name, "John Doe");
    assert_eq!(result.age_in_months, 300);

    conn.rollback().await?;
    Ok(())
}

// Test that using nonexistent columns from Output type produces compile error
// This test is commented out because it should fail to compile
/*
#[derive(Output)]
#[sql(table = OutputColumnsTestTable)]
#[sql(default = id)]
struct InvalidOutputColumnsTest {
    first_name: String,
    #[sql(select = nonexistent_column || " test")]
    invalid_field: String,
}
*/
