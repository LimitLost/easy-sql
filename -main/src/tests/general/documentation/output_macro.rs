#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as TestDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as TestDriver};

use super::super::macros::{ExprTestData, ExprTestTable, RelatedTestData, RelatedTestTable};
use crate::{DatabaseSetup, Output, PoolTransaction, table_join};
use easy_macros::{add_code, always_context};
use easy_sql_macros::query;

type ExampleTable = ExprTestTable;
type RelatedTable = RelatedTestTable;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(row.int_field, 42);
    assert_eq!(row.str_field, "ready");
    Ok(())
})]
#[docify::export_content]
async fn output_basic_example(mut conn: PoolTransaction<TestDriver>) -> anyhow::Result<()> {
    #[derive(Output)]
    #[sql(table = ExampleTable)]
    struct BasicOutput {
        int_field: i32,
        str_field: String,
    }

    let value = 42;
    let row: BasicOutput = query!(&mut conn,
        SELECT BasicOutput FROM ExampleTable WHERE ExampleTable.int_field = {value}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(row.int_field, 42);
    assert_eq!(row.int_plus_one, 43);
    Ok(())
})]
#[docify::export_content]
async fn output_custom_select_example(mut conn: PoolTransaction<TestDriver>) -> anyhow::Result<()> {
    #[derive(Output)]
    #[sql(table = ExampleTable)]
    struct CustomSelectOutput {
        int_field: i32,
        #[sql(select = int_field + 1)]
        int_plus_one: i32,
    }

    let row: CustomSelectOutput = query!(&mut conn,
        SELECT CustomSelectOutput FROM ExampleTable WHERE ExampleTable.int_field = 42
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(row.labeled, "echo!");
    Ok(())
})]
#[docify::export_content]
async fn output_custom_select_args_example(
    mut conn: PoolTransaction<TestDriver>,
) -> anyhow::Result<()> {
    #[derive(Output)]
    #[sql(table = ExampleTable)]
    struct CustomSelectArgsOutput {
        #[sql(select = str_field || {arg0})]
        labeled: String,
    }

    let suffix = "!";
    let row: CustomSelectArgsOutput = query!(&mut conn,
        SELECT CustomSelectArgsOutput({suffix}) FROM ExampleTable WHERE ExampleTable.int_field = 7
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].parent_name, "parent");
    assert_eq!(rows[0].child_data, "child_child");
    Ok(())
})]
#[docify::export_content]
async fn output_joined_fields_example(mut conn: PoolTransaction<TestDriver>) -> anyhow::Result<()> {
    table_join!(ExampleJoin | ExampleTable INNER JOIN RelatedTable ON ExampleTable.id = RelatedTable.parent_id);

    #[derive(Output)]
    #[sql(table = ExampleJoin)]
    struct JoinedOutput {
        #[sql(field = ExampleTable.str_field)]
        parent_name: String,
        #[sql(select = RelatedTable.data || "_child")]
        child_data: String,
    }

    let parent_id = 1;
    let rows: Vec<JoinedOutput> = query!(&mut conn,
        SELECT Vec<JoinedOutput> FROM ExampleJoin WHERE RelatedTable.parent_id = {parent_id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_output_basic_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    let data = ExprTestData {
        int_field: 42,
        str_field: "ready".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {data}).await?;

    output_basic_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_output_custom_select_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    let data = ExprTestData {
        int_field: 42,
        str_field: "custom".to_string(),
        bool_field: false,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {data}).await?;

    output_custom_select_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_output_custom_select_args_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    let data = ExprTestData {
        int_field: 7,
        str_field: "echo".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExampleTable VALUES {data}).await?;

    output_custom_select_args_example(conn).await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_output_joined_fields_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleTable>().await?;
    let mut conn = db.transaction().await?;

    RelatedTable::setup(&mut &mut conn).await?;

    let parent_data = ExprTestData {
        int_field: 5,
        str_field: "parent".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    let _parent: ExprTestData = query!(&mut conn,
        INSERT INTO ExampleTable VALUES {parent_data} RETURNING ExprTestData
    )
    .await?;

    let related_data = RelatedTestData {
        parent_id: 1,
        data: "child".to_string(),
    };
    query!(&mut conn, INSERT INTO RelatedTable VALUES {related_data}).await?;

    output_joined_fields_example(conn).await
}
