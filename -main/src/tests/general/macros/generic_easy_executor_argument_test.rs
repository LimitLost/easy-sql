use super::*;
use crate::EasyExecutor;
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::{query, query_lazy};
use futures::StreamExt;

type ExampleDriver = TestDriver;

#[always_context(skip(!))]
async fn generic_insert_and_fetch_rows(
    conn: &mut impl EasyExecutor<ExampleDriver>,
) -> anyhow::Result<Vec<ExprTestData>> {
    let first = expr_test_data(11, "generic-1", true, None);
    let second = expr_test_data(22, "generic-2", false, Some("n"));

    query!(*conn, INSERT INTO ExprTestTable VALUES {first}).await?;
    query!(*conn, INSERT INTO ExprTestTable VALUES {second}).await?;

    let rows: Vec<ExprTestData> = query!(*conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        ORDER BY ExprTestTable.id
    )
    .await?;

    Ok(rows)
}

#[always_context(skip(!))]
async fn generic_lazy_select_count(
    conn: &mut impl EasyExecutor<ExampleDriver>,
) -> anyhow::Result<usize> {
    let mut lazy = query_lazy!(<ExampleDriver>
        SELECT ExprTestData FROM ExprTestTable
        WHERE int_field >= 10
        ORDER BY ExprTestTable.id
    )?;

    let mut count = 0_usize;
    let mut stream = lazy.fetch(conn);
    while let Some(row) = stream.next().await {
        row.context("Failed to fetch row in generic lazy stream")?;
        count += 1;
    }

    Ok(count)
}

#[always_context(skip(!))]
async fn generic_lazy_fetch_multiple_times_with_same_conn(
    conn: &mut impl EasyExecutor<ExampleDriver>,
) -> anyhow::Result<(usize, usize)> {
    let mut first_lazy = query_lazy!(<ExampleDriver>
        SELECT ExprTestData FROM ExprTestTable
        WHERE int_field >= 10
        ORDER BY ExprTestTable.id
    )?;

    let first_count = {
        let mut count = 0_usize;
        let mut stream = first_lazy.fetch(&mut *conn);
        while let Some(row) = stream.next().await {
            row.context("Failed while fetching first lazy query rows")?;
            count += 1;
        }
        count
    };

    let mut second_lazy = query_lazy!(<ExampleDriver>
        SELECT ExprTestData FROM ExprTestTable
        WHERE int_field >= 20
        ORDER BY ExprTestTable.id
    )?;

    let second_count = {
        let mut count = 0_usize;
        let mut stream = second_lazy.fetch(conn);
        while let Some(row) = stream.next().await {
            row.context("Failed while fetching second lazy query rows")?;
            count += 1;
        }
        count
    };

    Ok((first_count, second_count))
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_generic_easy_executor_query_flow() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    let rows = generic_insert_and_fetch_rows(&mut conn).await?;

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].int_field, 11);
    assert_eq!(rows[0].str_field, "generic-1");
    assert_eq!(rows[1].int_field, 22);
    assert_eq!(rows[1].str_field, "generic-2");
    assert_eq!(rows[1].nullable_field, Some("n".to_string()));

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_generic_easy_executor_query_lazy_fetch_mut_flow() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(5, "skip", true, None),
            expr_test_data(10, "include-1", true, None),
            expr_test_data(20, "include-2", false, None),
        ],
    )
    .await?;

    let count = generic_lazy_select_count(&mut conn).await?;

    assert_eq!(count, 2);

    conn.rollback().await?;
    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_generic_easy_executor_query_lazy_multiple_fetches_same_conn() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(5, "below", true, None),
            expr_test_data(10, "ten", true, None),
            expr_test_data(20, "twenty", false, None),
            expr_test_data(30, "thirty", true, None),
        ],
    )
    .await?;

    let (first_count, second_count) =
        generic_lazy_fetch_multiple_times_with_same_conn(&mut conn).await?;

    assert_eq!(first_count, 3);
    assert_eq!(second_count, 2);

    conn.rollback().await?;
    Ok(())
}
