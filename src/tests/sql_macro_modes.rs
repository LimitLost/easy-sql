// Tests for the unified sql! macro with three modes:
// 1. Expression mode (like sql_where!)
// 2. Set clause mode (like sql_set!)
// 3. Select clauses mode (original sql! behavior)

use super::Database;
use crate::{Insert, Output, Table, Update};
use anyhow::Context;
use easy_macros::macros::always_context;
use sql_macros::{sql, sql_convenience};

#[derive(Table, Debug)]
#[sql(version = 1)]
#[sql(unique_id = "5058fa57-75a8-4db5-855b-7dffdb138c76")]
struct TestTableModes {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    field: i64,
    name: String,
}

#[derive(Insert, Update, Output, Debug)]
#[sql(table = TestTableModes)]
#[sql(default = id)]
struct TestOps {
    pub field: i64,
    pub name: String,
}

// =============== Expression Mode Tests ===============

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_expression_mode_simple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 42,
            name: "test".to_string(),
        },
    )
    .await?;

    // Expression mode: just id = 1 (no WHERE keyword)
    let row: TestOps = TestTableModes::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 42);
    assert_eq!(row.name, "test");

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_expression_mode_complex() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    for i in 1..=5 {
        TestTableModes::insert(
            &mut conn,
            &TestOps {
                field: i * 10,
                name: format!("name{}", i),
            },
        )
        .await?;
    }

    // Expression mode: complex expression with AND
    let rows: Vec<TestOps> =
        TestTableModes::select(&mut conn, sql!(id >= 2 AND field <= 40)).await?;
    assert_eq!(rows.len(), 3); // ids 2, 3, 4
    assert_eq!(rows[0].field, 20);
    assert_eq!(rows[2].field, 40);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_expression_mode_with_like() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 1,
            name: "hello".to_string(),
        },
    )
    .await?;
    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 2,
            name: "world".to_string(),
        },
    )
    .await?;

    // Expression mode: LIKE operator
    let rows: Vec<TestOps> = TestTableModes::select(&mut conn, sql!(name LIKE "h%")).await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].name, "hello");

    conn.rollback().await?;
    Ok(())
}

// =============== Set Clause Mode Tests ===============

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_set_clause_mode_simple() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 10,
            name: "old".to_string(),
        },
    )
    .await?;

    // Set clause mode: field = value, name = value (no WHERE)
    TestTableModes::update(&mut conn, sql!(field = 99, name = "new"), sql!(id = 1)).await?;

    let row: TestOps = TestTableModes::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 99);
    assert_eq!(row.name, "new");

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_set_clause_mode_arithmetic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 100,
            name: "test".to_string(),
        },
    )
    .await?;

    // Set clause mode with arithmetic: field = field + 50
    TestTableModes::update(&mut conn, sql!(field = field + 50), sql!(id = 1)).await?;

    let row: TestOps = TestTableModes::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 150);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_set_clause_mode_multiple_operations() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 10,
            name: "old".to_string(),
        },
    )
    .await?;

    // Set clause mode: multiple updates
    TestTableModes::update(
        &mut conn,
        sql!(field = field * 2, name = "updated"),
        sql!(id = 1),
    )
    .await?;

    let row: TestOps = TestTableModes::get(&mut conn, sql!(id = 1)).await?;
    assert_eq!(row.field, 20);
    assert_eq!(row.name, "updated");

    conn.rollback().await?;
    Ok(())
}

// =============== Select Clauses Mode Tests ===============

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_select_clauses_mode_where_only() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    for i in 1..=3 {
        TestTableModes::insert(
            &mut conn,
            &TestOps {
                field: i * 10,
                name: format!("name{}", i),
            },
        )
        .await?;
    }

    // Select clauses mode: WHERE clause
    let rows: Vec<TestOps> = TestTableModes::select(&mut conn, sql!(WHERE id > 1)).await?;
    assert_eq!(rows.len(), 2);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_select_clauses_mode_order_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 30,
            name: "c".to_string(),
        },
    )
    .await?;
    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 10,
            name: "a".to_string(),
        },
    )
    .await?;
    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 20,
            name: "b".to_string(),
        },
    )
    .await?;

    // Select clauses mode: ORDER BY
    let rows: Vec<TestOps> = TestTableModes::select(&mut conn, sql!(ORDER BY field ASC)).await?;
    assert_eq!(rows[0].field, 10);
    assert_eq!(rows[1].field, 20);
    assert_eq!(rows[2].field, 30);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_select_clauses_mode_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    for i in 1..=10 {
        TestTableModes::insert(
            &mut conn,
            &TestOps {
                field: i,
                name: format!("name{}", i),
            },
        )
        .await?;
    }

    // Select clauses mode: LIMIT
    let rows: Vec<TestOps> = TestTableModes::select(&mut conn, sql!(LIMIT 3)).await?;
    assert_eq!(rows.len(), 3);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_sql_select_clauses_mode_combined() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    for i in 1..=10 {
        TestTableModes::insert(
            &mut conn,
            &TestOps {
                field: i * 10,
                name: format!("name{}", i),
            },
        )
        .await?;
    }

    // Select clauses mode: WHERE + ORDER BY + LIMIT
    let rows: Vec<TestOps> = TestTableModes::select(
        &mut conn,
        sql!(WHERE field >= 30 ORDER BY field DESC LIMIT 3),
    )
    .await?;
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].field, 100); // DESC order: 100, 90, 80
    assert_eq!(rows[1].field, 90);
    assert_eq!(rows[2].field, 80);

    conn.rollback().await?;
    Ok(())
}

// =============== Mode Detection Edge Cases ===============

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_mode_detection_field_vs_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    TestTableModes::insert(
        &mut conn,
        &TestOps {
            field: 42,
            name: "test".to_string(),
        },
    )
    .await?;

    // This should be detected as expression mode (no WHERE keyword)
    let row: TestOps = TestTableModes::get(&mut conn, sql!(field = 42)).await?;
    assert_eq!(row.field, 42);

    // This should be detected as select clauses mode (has WHERE keyword)
    let rows: Vec<TestOps> = TestTableModes::select(&mut conn, sql!(WHERE field = 42)).await?;
    assert_eq!(rows.len(), 1);

    conn.rollback().await?;
    Ok(())
}

#[sql_convenience]
#[always_context]
#[tokio::test]
async fn test_backwards_compatibility() -> anyhow::Result<()> {
    // Verify that existing code using WHERE, ORDER BY, LIMIT still works
    let db = Database::setup_for_testing::<TestTableModes>().await?;
    let mut conn = db.transaction().await?;

    for i in 1..=5 {
        TestTableModes::insert(
            &mut conn,
            &TestOps {
                field: i,
                name: format!("name{}", i),
            },
        )
        .await?;
    }

    // Original sql! macro syntax should still work
    let rows: Vec<TestOps> =
        TestTableModes::select(&mut conn, sql!(WHERE id >= 2 ORDER BY id LIMIT 2)).await?;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].field, 2);
    assert_eq!(rows[1].field, 3);

    conn.rollback().await?;
    Ok(())
}
