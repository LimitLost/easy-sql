#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as ExampleDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as ExampleDriver};

use crate::{Insert, Table, Transaction};
use easy_macros::{add_code, always_context};
use sql_macros::query;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct DocInsertTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    #[sql(default = "guest".to_string())]
    role: String,
    active: bool,
    nickname: Option<String>,
}

type ExampleTable = DocInsertTable;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let rows: Vec<ExampleTable> = query!(&mut conn,
        SELECT Vec<ExampleTable> FROM ExampleTable WHERE ExampleTable.name = "sam"
    )
    .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].role, "admin");
    Ok(())
})]
#[docify::export_content]
async fn insert_basic_example(mut conn: Transaction<'_, ExampleDriver>) -> anyhow::Result<()> {
    #[derive(Insert)]
    #[sql(table = ExampleTable)]
    struct ExampleInsert {
        id: i32,
        name: String,
        role: String,
        active: bool,
        nickname: Option<String>,
    }

    let data = ExampleInsert {
        id: 4,
        name: "sam".to_string(),
        role: "admin".to_string(),
        active: true,
        nickname: None,
    };

    query!(&mut conn, INSERT INTO ExampleTable VALUES {data}).await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let rows: Vec<ExampleTable> = query!(&mut conn,
        SELECT Vec<ExampleTable> FROM ExampleTable WHERE ExampleTable.name = "pat"
    )
    .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].role, "guest");
    Ok(())
})]
#[docify::export_content]
async fn insert_defaults_example(mut conn: Transaction<'_, ExampleDriver>) -> anyhow::Result<()> {
    #[derive(Insert)]
    #[sql(table = ExampleTable)]
    #[sql(default = id, role)]
    struct ExampleInsert {
        name: String,
        active: bool,
        nickname: Option<String>,
    }

    let data = ExampleInsert {
        name: "pat".to_string(),
        active: false,
        nickname: Some("Pat".to_string()),
    };

    query!(&mut conn, INSERT INTO ExampleTable VALUES {&data}).await?;
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_insert_basic_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let conn = db.transaction().await?;

    insert_basic_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_insert_defaults_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let conn = db.transaction().await?;

    insert_defaults_example(conn).await
}
