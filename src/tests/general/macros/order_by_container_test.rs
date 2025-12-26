//! Test that ExprTestData.field syntax works with Vec<ExprTestData> and Option<ExprTestData>

use super::*;
use anyhow::Result;
use sql_macros::query;

#[tokio::test]
async fn test_vec_container_with_qualified_output_syntax() -> Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(20, "b", false, None),
        ],
    )
    .await?;

    // ExprTestData.int_field should work with Vec<ExprTestData> output type
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData>
        FROM ExprTestTable
        WHERE ExprTestData.int_field > 5
        ORDER BY ExprTestData.int_field ASC
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 30);

    conn.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn test_option_container_with_qualified_output_syntax() -> Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, expr_test_data(42, "test", true, None)).await?;

    // ExprTestData.int_field should work with Option<ExprTestData> output type
    let result: Option<ExprTestData> = query!(&mut conn,
        SELECT Option<ExprTestData>
        FROM ExprTestTable
        WHERE ExprTestData.int_field = 42
        ORDER BY ExprTestData.str_field
    )
    .await?;

    assert!(result.is_some());
    assert_eq!(result.unwrap().int_field, 42);

    conn.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn test_direct_output_with_qualified_syntax() -> Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, expr_test_data(42, "test", true, None)).await?;

    // ExprTestData.int_field should work with direct ExprTestData output type
    let result: ExprTestData = query!(&mut conn,
        SELECT ExprTestData
        FROM ExprTestTable
        WHERE ExprTestData.int_field = 42
        ORDER BY ExprTestData.str_field
    )
    .await?;

    assert_eq!(result.int_field, 42);

    conn.rollback().await?;
    Ok(())
}
