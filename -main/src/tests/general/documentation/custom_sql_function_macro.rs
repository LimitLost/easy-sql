#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as ExampleDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as ExampleDriver};

use super::super::macros::{
    ExprTestData, ExprTestTable, expr_test_data, insert_multiple_test_data,
};
use crate::{Transaction, custom_sql_function};
use easy_macros::{add_code, always_context};
use easy_sql_macros::query;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].str_field, "hello");
    Ok(())
})]
#[allow(non_local_definitions)]
#[docify::export_content]
async fn custom_sql_function_basic_example(
    mut conn: Transaction<'_, ExampleDriver>,
) -> anyhow::Result<()> {
    custom_sql_function!(Capitalize; "UPPER"; 1);

    let rows: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE Capitalize(str_field) = "HELLO"
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(two_args.len(), 1);
    assert_eq!(three_args.len(), 1);
    Ok(())
})]
#[allow(non_local_definitions)]
#[docify::export_content]
async fn custom_sql_function_multiple_args_example(
    mut conn: Transaction<'_, ExampleDriver>,
) -> anyhow::Result<()> {
    custom_sql_function!(SubstrSlice; "SUBSTR"; 2 | 3);

    let two_args: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE substrSlice(str_field, 2) = "ello"
    )
    .await?;

    let three_args: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE substrSLICE(str_field, 2, 3) = "ell"
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].str_field, "hello");
    Ok(())
})]
#[allow(non_local_definitions)]
#[docify::export_content]
async fn custom_sql_function_any_args_example(
    mut conn: Transaction<'_, ExampleDriver>,
) -> anyhow::Result<()> {
    custom_sql_function!(CoalesceAnyDoc; "COALESCE"; Any);

    let rows: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE CoalesceAnyDoc(nullable_field, str_field, "fallback") = "hello"
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_custom_sql_function_basic_example() -> anyhow::Result<()> {
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

    custom_sql_function_basic_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_custom_sql_function_multiple_args_example() -> anyhow::Result<()> {
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

    custom_sql_function_multiple_args_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_custom_sql_function_any_args_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(1, "hello", true, None),
            expr_test_data(2, "world", true, Some("alt")),
        ],
    )
    .await?;

    custom_sql_function_any_args_example(conn).await
}
