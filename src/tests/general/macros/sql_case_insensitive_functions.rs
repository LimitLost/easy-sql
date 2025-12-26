// Test that SQL functions can be called with any case
use super::*;
use easy_macros::always_context;
use sql_macros::query;

#[derive(Table, Debug, Clone)]
#[sql(version = 1)]
#[sql(unique_id = "5d28c663-9bdd-40dd-a1c8-3dd40f567974")]
pub struct CaseFunctionTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub name: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = CaseFunctionTestTable)]
#[sql(default = id)]
pub struct CaseFunctionTestData {
    pub name: String,
}

/// Test UPPER function in all uppercase
#[always_context(skip(!))]
#[tokio::test]
async fn test_upper_uppercase() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CaseFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CaseFunctionTestData {
        name: "hello".to_string(),
    };
    query!(&mut conn, INSERT INTO CaseFunctionTestTable VALUES {data}).await?;

    let result: CaseFunctionTestData = query!(&mut conn,
        SELECT CaseFunctionTestData FROM CaseFunctionTestTable
        WHERE UPPER(name) = "HELLO"
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Test UPPER function in lowercase
#[always_context(skip(!))]
#[tokio::test]
async fn test_upper_lowercase() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CaseFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CaseFunctionTestData {
        name: "hello".to_string(),
    };
    query!(&mut conn, INSERT INTO CaseFunctionTestTable VALUES {data}).await?;

    let result: CaseFunctionTestData = query!(&mut conn,
        SELECT CaseFunctionTestData FROM CaseFunctionTestTable
        WHERE upper(name) = "HELLO"
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Test UPPER function in mixed case
#[always_context(skip(!))]
#[tokio::test]
async fn test_upper_mixedcase() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CaseFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CaseFunctionTestData {
        name: "hello".to_string(),
    };
    query!(&mut conn, INSERT INTO CaseFunctionTestTable VALUES {data}).await?;

    let result: CaseFunctionTestData = query!(&mut conn,
        SELECT CaseFunctionTestData FROM CaseFunctionTestTable
        WHERE Upper(name) = "HELLO"
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Test LENGTH function in all uppercase
#[always_context(skip(!))]
#[tokio::test]
async fn test_length_uppercase() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CaseFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CaseFunctionTestData {
        name: "hello".to_string(),
    };
    query!(&mut conn, INSERT INTO CaseFunctionTestTable VALUES {data}).await?;

    let result: CaseFunctionTestData = query!(&mut conn,
        SELECT CaseFunctionTestData FROM CaseFunctionTestTable
        WHERE LENGTH(name) = 5
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Test LENGTH function in lowercase
#[always_context(skip(!))]
#[tokio::test]
async fn test_length_lowercase() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CaseFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CaseFunctionTestData {
        name: "hello".to_string(),
    };
    query!(&mut conn, INSERT INTO CaseFunctionTestTable VALUES {data}).await?;

    let result: CaseFunctionTestData = query!(&mut conn,
        SELECT CaseFunctionTestData FROM CaseFunctionTestTable
        WHERE  length(name) = 5
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}

/// Test LENGTH function in mixed case
#[always_context(skip(!))]
#[tokio::test]
async fn test_length_mixedcase() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<CaseFunctionTestTable>().await?;
    let mut conn = db.transaction().await?;

    let data = CaseFunctionTestData {
        name: "hello".to_string(),
    };
    query!(&mut conn, INSERT INTO CaseFunctionTestTable VALUES {data}).await?;

    let result: CaseFunctionTestData = query!(&mut conn,
        SELECT CaseFunctionTestData FROM CaseFunctionTestTable
        WHERE LeNgTh(name) = 5
    )
    .await?;

    assert_eq!(result.name, "hello");
    conn.rollback().await?;
    Ok(())
}
