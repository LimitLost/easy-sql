//! Test ORDER BY with Output type columns when use_output_columns feature is enabled

use super::*;
use anyhow::Result;
use easy_sql_macros::query;

#[tokio::test]
async fn test_order_by_with_output_column_qualified_syntax() -> Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(20, "b", false, None),
        ],
    )
    .await?;

    // Test ORDER BY with qualified Output column syntax: ExprTestData.int_field
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable ORDER BY ExprTestData.int_field ASC
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 30);

    // Test ORDER BY DESC
    let results_desc: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable ORDER BY ExprTestData.int_field DESC
    )
    .await?;

    assert_eq!(results_desc.len(), 3);
    assert_eq!(results_desc[0].int_field, 30);
    assert_eq!(results_desc[1].int_field, 20);
    assert_eq!(results_desc[2].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

#[cfg(feature = "use_output_columns")]
#[tokio::test]
async fn test_order_by_with_bare_output_column() -> Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(20, "b", false, None),
        ],
    )
    .await?;

    // With use_output_columns feature, bare column names should validate against Output type
    // When selecting Vec<ExprTestData>, bare columns won't work because Vec doesn't have fields
    // This test documents the expected behavior - use qualified syntax instead

    // This would fail to compile with use_output_columns because Vec<ExprTestData> has no int_field:
    // let results: Vec<ExprTestData> = query!(&mut conn,
    //     SELECT Vec<ExprTestData> FROM ExprTestTable ORDER BY int_field ASC
    // )
    // .await?;

    // Instead, users should use:
    // 1. Qualified syntax with Output type: ExprTestData.int_field
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable ORDER BY ExprTestData.int_field ASC
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);

    // 2. Or qualified syntax with Table type: ExprTestTable.int_field
    let results2: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable ORDER BY ExprTestTable.int_field ASC
    )
    .await?;

    assert_eq!(results2.len(), 3);
    assert_eq!(results2[0].int_field, 10);

    conn.rollback().await?;
    Ok(())
}

#[tokio::test]
async fn test_order_by_with_table_column_qualified_syntax() -> Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    // Insert test data
    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", false, None),
            expr_test_data(30, "c", true, None),
            expr_test_data(20, "b", false, None),
        ],
    )
    .await?;

    // Test ORDER BY with qualified Table column syntax: ExprTestTable.int_field
    // This should always work regardless of feature flag
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable ORDER BY ExprTestTable.int_field ASC
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].int_field, 10);
    assert_eq!(results[1].int_field, 20);
    assert_eq!(results[2].int_field, 30);

    conn.rollback().await?;
    Ok(())
}
