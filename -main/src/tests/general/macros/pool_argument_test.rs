use super::*;
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::{query, query_lazy};
use futures::StreamExt;

type SqlxPool = sqlx::Pool<<TestDriver as crate::Driver>::InternalDriver>;

#[always_context(skip(!))]
async fn pool_query_insert_and_read(mut conn: &SqlxPool) -> anyhow::Result<ExprTestData> {
    let data = expr_test_data(77, "pool-fn", true, Some("from-pool-fn"));
    query!(conn, INSERT INTO ExprTestTable VALUES {data}).await?;

    let row: ExprTestData = query!(conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;

    Ok(row)
}

#[always_context(skip(!))]
async fn pool_query_lazy_collect(conn: &SqlxPool) -> anyhow::Result<Vec<ExprTestData>> {
    let mut lazy = query_lazy!(
        SELECT ExprTestData FROM ExprTestTable WHERE int_field >= 20 ORDER BY ExprTestTable.id
    )?;

    let mut rows = Vec::new();
    let mut stream = lazy.fetch(conn);
    while let Some(row) = stream.next().await {
        rows.push(row.context("Failed to fetch row from pool lazy stream")?);
    }

    Ok(rows)
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_with_pool_function_argument() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ExprTestTable>().await?;
    let pool = pool_resource.pool();

    let row = pool_query_insert_and_read(pool).await?;

    assert_eq!(row.int_field, 77);
    assert_eq!(row.str_field, "pool-fn");
    assert_eq!(row.nullable_field, Some("from-pool-fn".to_string()));

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_query_lazy_with_pool_function_argument() -> anyhow::Result<()> {
    let pool_resource = setup_sqlx_pool_for_testing::<ExprTestTable>().await?;
    let mut pool = pool_resource.pool();

    let seed = vec![
        expr_test_data(10, "a", true, None),
        expr_test_data(20, "b", false, None),
        expr_test_data(30, "c", true, None),
    ];
    query!(pool, INSERT INTO ExprTestTable VALUES {seed}).await?;

    let rows = pool_query_lazy_collect(pool).await?;

    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].int_field, 20);
    assert_eq!(rows[1].int_field, 30);

    Ok(())
}
