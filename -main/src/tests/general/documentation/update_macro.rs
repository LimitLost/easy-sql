#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as TestDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as TestDriver};

use crate::{Insert, Table, Transaction, Update};
use easy_macros::{add_code, always_context};
use easy_sql_macros::query;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct DocUpdateTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    active: bool,
    nickname: Option<String>,
}

#[derive(Insert)]
#[sql(table = DocUpdateTable)]
#[sql(default = id)]
struct DocUpdateInsert {
    name: String,
    active: bool,
    nickname: Option<String>,
}

type ExampleTable = DocUpdateTable;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let row: ExampleTable = query!(&mut conn,
        SELECT ExampleTable FROM ExampleTable WHERE ExampleTable.id = 1
    )
    .await?;
    assert_eq!(row.name, "updated");
    assert!(!row.active);
    Ok(())
})]
#[docify::export_content]
async fn update_basic_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    #[derive(Update)]
    #[sql(table = ExampleTable)]
    struct UpdateData {
        name: String,
        active: bool,
    }

    let id = 1;
    let update = UpdateData {
        name: "updated".to_string(),
        active: false,
    };

    query!(&mut conn,
        UPDATE ExampleTable SET {update} WHERE ExampleTable.id = {id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let row: ExampleTable = query!(&mut conn,
        SELECT ExampleTable FROM ExampleTable WHERE ExampleTable.id = 1
    )
    .await?;
    assert_eq!(row.nickname, None);
    Ok(())
})]
#[docify::export_content]
async fn update_partial_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    #[derive(Update)]
    #[sql(table = ExampleTable)]
    struct UpdateNickname {
        nickname: Option<String>,
    }

    let id = 1;
    let update = UpdateNickname { nickname: None };

    query!(&mut conn,
        UPDATE ExampleTable SET {&update} WHERE ExampleTable.id = {id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_update_basic_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    let data = DocUpdateInsert {
        name: "original".to_string(),
        active: true,
        nickname: Some("first".to_string()),
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {data}).await?;

    update_basic_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_update_partial_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    let data = DocUpdateInsert {
        name: "original".to_string(),
        active: true,
        nickname: Some("before".to_string()),
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {data}).await?;

    update_partial_example(conn).await
}
