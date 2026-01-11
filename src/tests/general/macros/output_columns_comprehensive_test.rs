// Comprehensive tests for the use_output_columns feature
// This feature allows referencing Output type fields in custom select expressions
// Enable with: cargo test --features use_output_columns

use crate::Output;
use crate::tests::general::Database;
use crate::tests::general::macros::{ExprTestData, ExprTestTable};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::query;

mod output_columns_tests {
    use super::*;

    // Test 1: Basic Output column self-reference
    #[derive(Output, Debug, Clone, PartialEq)]
    #[sql(table = ExprTestTable)]
    #[sql(default = id)]
    struct BasicOutputRef {
        int_field: i32,
        str_field: String,
        #[sql(select = ExprTestTable.str_field || " (" || int_field || ")")]
        combined: String,
    }

    // Test 2: Complex nested expressions
    #[derive(Output, Debug, Clone, PartialEq)]
    #[sql(table = ExprTestTable)]
    #[sql(default = id)]
    struct ComplexExpression {
        int_field: i32,
        str_field: String,
        #[sql(select = "Value: " || ExprTestTable.str_field || " = " || ExprTestTable.int_field)]
        full_info: String,
    }

    // Test 3: Mathematical operations with Output references
    #[derive(Output, Debug, Clone, PartialEq)]
    #[sql(table = ExprTestTable)]
    #[sql(default = id)]
    struct MathOperations {
        int_field: i32,
        #[sql(select = ExprTestTable.int_field * 2)]
        doubled: i32,
        #[sql(select = ExprTestTable.int_field + 10)]
        plus_ten: i32,
    }

    // Test 4: Mixed column references (Output.column and Table.column)
    #[derive(Output, Debug, Clone, PartialEq)]
    #[sql(table = ExprTestTable)]
    #[sql(default = id)]
    struct MixedColumnReferences {
        str_field: String,
        int_field: i32,
        #[sql(select = str_field || " -> " || ExprTestTable.int_field)]
        description: String,
    }

    #[always_context(skip(!))]
    #[tokio::test]
    async fn test_basic_output_column_reference() -> anyhow::Result<()> {
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        query!(&mut conn, INSERT INTO ExprTestTable VALUES {
            ExprTestData {
                int_field: 42,
                str_field: "test".to_string(),
                bool_field: true,
                nullable_field: None,
            }
        })
        .await?;

        let result: BasicOutputRef = query!(&mut conn,
            SELECT BasicOutputRef FROM ExprTestTable WHERE ExprTestTable.id = 1
        )
        .await?;

        assert_eq!(result.int_field, 42);
        assert_eq!(result.str_field, "test");
        assert_eq!(result.combined, "test (42)");

        conn.rollback().await?;
        Ok(())
    }

    #[always_context(skip(!))]
    #[tokio::test]
    async fn test_complex_nested_expression() -> anyhow::Result<()> {
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        query!(&mut conn, INSERT INTO ExprTestTable VALUES {
            ExprTestData {
                int_field: 100,
                str_field: "data".to_string(),
                bool_field: false,
                nullable_field: Some("extra".to_string()),
            }
        })
        .await?;

        let result: ComplexExpression = query!(&mut conn,
            SELECT ComplexExpression FROM ExprTestTable WHERE ExprTestTable.id = 1
        )
        .await?;

        assert_eq!(result.int_field, 100);
        assert_eq!(result.str_field, "data");
        assert_eq!(result.full_info, "Value: data = 100");

        conn.rollback().await?;
        Ok(())
    }

    #[always_context(skip(!))]
    #[tokio::test]
    async fn test_math_operations() -> anyhow::Result<()> {
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        query!(&mut conn, INSERT INTO ExprTestTable VALUES {
            ExprTestData {
                int_field: 25,
                str_field: "math".to_string(),
                bool_field: true,
                nullable_field: None,
            }
        })
        .await?;

        let result: MathOperations = query!(&mut conn,
            SELECT MathOperations FROM ExprTestTable WHERE ExprTestTable.id = 1
        )
        .await?;

        assert_eq!(result.int_field, 25);
        assert_eq!(result.doubled, 50);
        assert_eq!(result.plus_ten, 35);

        conn.rollback().await?;
        Ok(())
    }

    #[always_context(skip(!))]
    #[tokio::test]
    async fn test_mixed_column_references() -> anyhow::Result<()> {
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        query!(&mut conn, INSERT INTO ExprTestTable VALUES {
            ExprTestData {
                int_field: 77,
                str_field: "mixed".to_string(),
                bool_field: true,
                nullable_field: None,
            }
        })
        .await?;

        let result: MixedColumnReferences = query!(&mut conn,
            SELECT MixedColumnReferences FROM ExprTestTable WHERE ExprTestTable.id = 1
        )
        .await?;

        assert_eq!(result.str_field, "mixed");
        assert_eq!(result.int_field, 77);
        assert_eq!(result.description, "mixed -> 77");

        conn.rollback().await?;
        Ok(())
    }

    #[always_context(skip(!))]
    #[tokio::test]
    async fn test_where_clause_uses_table_columns() -> anyhow::Result<()> {
        let db = Database::setup_for_testing::<ExprTestTable>().await?;
        let mut conn = db.transaction().await?;

        query!(&mut conn, INSERT INTO ExprTestTable VALUES {
            ExprTestData {
                int_field: 99,
                str_field: "query".to_string(),
                bool_field: true,
                nullable_field: None,
            }
        })
        .await?;

        // WHERE clause should use Table columns, not Output columns
        let result: BasicOutputRef = query!(&mut conn,
            SELECT BasicOutputRef FROM ExprTestTable
            WHERE ExprTestTable.str_field = "query" AND ExprTestTable.int_field > 50
        )
        .await?;

        assert_eq!(result.str_field, "query");
        assert_eq!(result.combined, "query (99)");

        conn.rollback().await?;
        Ok(())
    }
}

// Note: When the "use_output_columns" feature is disabled,
// the existing test in output_columns_in_custom_select_test.rs will fail to compile
// with proper error messages, which is the expected behavior.
