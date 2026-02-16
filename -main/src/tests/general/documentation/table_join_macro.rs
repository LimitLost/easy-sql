#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as ExampleDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as ExampleDriver};

use super::super::macros::{ExprTestData, ExprTestTable, RelatedTestData, RelatedTestTable};
use crate::{DatabaseSetup, Output, PoolTransaction, table_join};
use easy_macros::{add_code, always_context};
use easy_sql_macros::query;

type ExampleTable = ExprTestTable;
type RelatedTable = RelatedTestTable;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].parent_name, "parent");
    assert_eq!(rows[0].child_data, "child");
    Ok(())
})]
#[docify::export_content]
async fn table_join_basic_example(mut conn: PoolTransaction<ExampleDriver>) -> anyhow::Result<()> {
    table_join!(ExampleJoin | ExampleTable INNER JOIN RelatedTable ON ExampleTable.id = RelatedTable.parent_id);

    #[derive(Output)]
    #[sql(table = ExampleJoin)]
    struct JoinedOutput {
        #[sql(field = ExampleTable.str_field)]
        parent_name: String,
        #[sql(field = RelatedTable.data)]
        child_data: String,
    }

    let parent_id = 1;
    let rows: Vec<JoinedOutput> = query!(&mut conn,
        SELECT Vec<JoinedOutput> FROM ExampleJoin
        WHERE RelatedTable.parent_id = {parent_id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].parent_name, "parent");
    assert_eq!(rows[0].child_data.as_deref(), Some("child"));
    assert_eq!(rows[0].multi_table_data.as_deref(), Some("parentchild"));
    assert_eq!(rows[1].parent_name, "lonely");
    assert!(rows[1].child_data.is_none());
    Ok(())
})]
#[docify::export_content]
async fn table_join_left_example(mut conn: PoolTransaction<ExampleDriver>) -> anyhow::Result<()> {
    table_join!(<ExampleDriver> ExampleLeftJoin | ExampleTable LEFT JOIN RelatedTable ON ExampleTable.id = RelatedTable.parent_id);

    #[derive(Output)]
    #[sql(table = ExampleLeftJoin)]
    struct LeftJoinOutput {
        #[sql(select = ExampleTable.str_field || RelatedTable.data)]
        multi_table_data: Option<String>,
        #[sql(field = ExampleTable.str_field)]
        parent_name: String,
        #[sql(field = RelatedTable.data)]
        child_data: Option<String>,
    }

    let rows: Vec<LeftJoinOutput> = query!(&mut conn,
        SELECT Vec<LeftJoinOutput> FROM ExampleLeftJoin
        ORDER BY ExampleTable.id ASC
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_table_join_basic_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    RelatedTable::setup(&mut &mut conn).await?;

    let parent_data = ExprTestData {
        int_field: 1,
        str_field: "parent".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {parent_data}).await?;

    let related_data = RelatedTestData {
        parent_id: 1,
        data: "child".to_string(),
    };
    query!(&mut conn, INSERT INTO RelatedTable VALUES {related_data}).await?;

    table_join_basic_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_table_join_left_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    RelatedTable::setup(&mut &mut conn).await?;

    let parent_data = ExprTestData {
        int_field: 10,
        str_field: "parent".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {parent_data}).await?;

    let lonely_data = ExprTestData {
        int_field: 20,
        str_field: "lonely".to_string(),
        bool_field: false,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {lonely_data}).await?;

    let related_data = RelatedTestData {
        parent_id: 1,
        data: "child".to_string(),
    };
    query!(&mut conn, INSERT INTO RelatedTable VALUES {related_data}).await?;

    table_join_left_example(conn).await
}
