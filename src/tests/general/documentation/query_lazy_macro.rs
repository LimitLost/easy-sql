#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as ExampleDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as ExampleDriver};

use super::super::macros::{
    ExprTestData, ExprTestTable, default_expr_test_data, expr_test_data, insert_multiple_test_data,
    insert_test_data,
};
use crate::{EasyExecutor, Insert, Output, Table, Transaction};
use anyhow::Context;
use easy_macros::{add_code, always_context};
use futures::StreamExt;
use sql_macros::query_lazy;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct DocLazyTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    column: i32,
}

#[derive(Insert, Output, Debug, Clone, PartialEq)]
#[sql(table = DocLazyTable)]
#[sql(default = id)]
struct DocLazyData {
    column: i32,
}

type OutputType = DocLazyData;
type TableType = DocLazyTable;
type Sqlite = ExampleDriver;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(row.column, 42);
    Ok(())
})]
#[docify::export_content]
async fn query_lazy_basic_example(mut conn: Transaction<'_, ExampleDriver>) -> anyhow::Result<()> {
    let mut lazy = query_lazy!(<Sqlite> SELECT OutputType FROM TableType WHERE column = 42)?;
    let mut stream = lazy.fetch(&mut conn);
    let row = stream.next().await.context("Expected at least one row")?;
    let row = row.context("Failed to fetch row")?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 2);
    Ok(())
})]
#[docify::export_content]
async fn query_lazy_streaming_example(
    mut conn: Transaction<'_, ExampleDriver>,
) -> anyhow::Result<()> {
    let min_val = 10;
    let mut lazy = query_lazy!(
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.int_field > {min_val}
    )?;

    let mut rows = Vec::new();
    let mut stream = lazy.fetch(&mut conn);
    while let Some(row) = stream.next().await {
        rows.push(row.context("Failed to fetch row")?);
    }
}

#[always_context(skip(!))]
#[no_context]
async fn external_generic_executor_example(
    conn: &mut impl EasyExecutor<ExampleDriver>,
) -> anyhow::Result<()> {
    #[docify::export]
    async fn generic_executor_example(
        conn: &mut impl EasyExecutor<ExampleDriver>,
    ) -> anyhow::Result<()> {
        let mut lazy = query_lazy!(SELECT ExprTestData FROM ExprTestTable)?;
        let mut stream = lazy.fetch_mut(conn);
        let maybe = stream.next().await.transpose()?;
        assert!(maybe.is_some());
        Ok(())
    }
    generic_executor_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_query_lazy_basic_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<DocLazyTable>().await?;
    let mut conn = db.transaction().await?;

    let data = DocLazyData { column: 42 };
    let mut lazy_insert =
        query_lazy!(<Sqlite> INSERT INTO DocLazyTable VALUES {data} RETURNING DocLazyData)?;
    let mut stream = lazy_insert.fetch(&mut conn);
    let inserted = stream
        .next()
        .await
        .context("Expected INSERT to return a row")?;
    let _inserted = inserted.context("Failed to fetch inserted row")?;
    drop(stream);

    query_lazy_basic_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_query_lazy_streaming_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(5, "low", false, None),
            expr_test_data(20, "mid", true, None),
            expr_test_data(30, "high", true, None),
        ],
    )
    .await?;

    query_lazy_streaming_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_query_lazy_generic_executor_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;

    let mut conn_ref = &mut conn;
    external_generic_executor_example(&mut conn_ref).await?;
    Ok(())
}
