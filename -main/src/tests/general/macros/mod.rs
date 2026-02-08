// Test module for query! and query_lazy! macros

use super::{Database, TestDriver};
use crate::{Insert, Output, Table, Update};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;

// ====================
// Shared Test Tables
// ====================

/// Main test table for SQL expression testing
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct ExprTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub int_field: i32,
    pub str_field: String,
    pub bool_field: bool,
    pub nullable_field: Option<String>,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = ExprTestTable)]
#[sql(default = id)]
pub struct ExprTestData {
    pub int_field: i32,
    pub str_field: String,
    pub bool_field: bool,
    pub nullable_field: Option<String>,
}

/// Table for testing relationships and joins
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct RelatedTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    #[sql(foreign_key = ExprTestTable)]
    pub parent_id: i32,
    pub data: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = RelatedTestTable)]
#[sql(default = id)]
pub struct RelatedTestData {
    pub parent_id: i32,
    pub data: String,
}

/// Table for testing numeric operations
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct NumericTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub int_val: i32,
    pub float_val: Option<f64>,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = NumericTestTable)]
#[sql(default = id)]
pub struct NumericTestData {
    pub int_val: i32,
    pub float_val: Option<f64>,
}

// ====================
// Test Utilities
// ====================

/// Helper to create test data with default values
pub fn default_expr_test_data() -> ExprTestData {
    ExprTestData {
        int_field: 42,
        str_field: "test".to_string(),
        bool_field: true,
        nullable_field: None,
    }
}

/// Helper to create test data with custom values
pub fn expr_test_data(
    int_field: i32,
    str_field: &str,
    bool_field: bool,
    nullable_field: Option<&str>,
) -> ExprTestData {
    ExprTestData {
        int_field,
        str_field: str_field.to_string(),
        bool_field,
        nullable_field: nullable_field.map(|s| s.to_string()),
    }
}

/// Helper to insert test data and return its ID
#[always_context(skip(!))]
pub async fn insert_test_data(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    data: ExprTestData,
) -> anyhow::Result<()> {
    query!(conn, INSERT INTO ExprTestTable VALUES {data})
        .await
        .context("Failed to insert test data")?;
    Ok(())
}

/// Helper to insert multiple test records
#[always_context(skip(!))]
pub async fn insert_multiple_test_data(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    data_vec: Vec<ExprTestData>,
) -> anyhow::Result<()> {
    query!(conn, INSERT INTO ExprTestTable VALUES {data_vec}).await?;
    Ok(())
}

// ====================
// Sub-modules
// ====================

mod custom_select;
mod custom_select_compile_fail;
mod order_by_container_test;
mod order_by_output_columns_test;
mod output_columns_comprehensive_test;
mod output_columns_in_custom_select_test;
mod query_lazy_macro;
mod query_macro;
mod sql_expressions;

mod custom_select_validation_test;
mod sql_case_insensitive_functions;
mod sql_functions;
mod custom_sql_functions;

mod table_join;
