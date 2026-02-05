#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as TestDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as TestDriver};

use super::super::macros::{
    ExprTestData, ExprTestTable, RelatedTestData, RelatedTestTable, default_expr_test_data,
    expr_test_data, insert_multiple_test_data, insert_test_data,
};
use crate::{DatabaseSetup, Insert, Output, Table, Transaction, custom_sql_function, table_join};
use easy_macros::{add_code, always_context};
use sql_macros::query;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct DocBasicTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    column: i32,
}

#[derive(Insert, Output, Debug, Clone, PartialEq)]
#[sql(table = DocBasicTable)]
#[sql(default = id)]
struct DocBasicData {
    column: i32,
}

type OutputType = DocBasicData;
type TableType = DocBasicTable;
type Sqlite = TestDriver;

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(result.column, value);
    Ok(())
})]
#[docify::export_content]
async fn query_basic_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let value = 42;
    let result =
        query!(<Sqlite> &mut conn, SELECT OutputType From TableType where column = {value}).await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(one.int_field, 42);
    assert_eq!(many.len(), 1);
    assert!(maybe.is_some());
    Ok(())
})]
#[docify::export_content]
async fn select_examples(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let id = 1;
    let one: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = {id}
    )
    .await?;
    let many: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE ExprTestTable.int_field > 0
    )
    .await?;
    let maybe: Option<ExprTestData> = query!(&mut conn,
        SELECT Option<ExprTestData> FROM ExprTestTable WHERE ExprTestTable.id < {id+1}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].int_field, 30);
    assert_eq!(results[1].int_field, 20);
    Ok(())
})]
#[docify::export_content]
async fn select_output_columns_example(
    mut conn: Transaction<'_, TestDriver>,
) -> anyhow::Result<()> {
    let min_val = 10;
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE ExprTestTable.int_field > {min_val} AND ExprTestData.bool_field = true
        ORDER BY int_field DESC
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let inserted: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE str_field = "ready"
    )
    .await?;
    assert_eq!(inserted.len(), 1);
    Ok(())
})]
#[docify::export_content]
async fn insert_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let data = ExprTestData {
        int_field: 42,
        str_field: "ready".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    query!(&mut conn, INSERT INTO ExprTestTable VALUES {data}).await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(inserted.int_field, 7);
    assert_eq!(inserted.str_field, "added");
    Ok(())
})]
#[docify::export_content]
async fn insert_returning_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let data = ExprTestData {
        int_field: 7,
        str_field: "added".to_string(),
        bool_field: false,
        nullable_field: None,
    };
    let inserted: ExprTestData = query!(&mut conn,
        INSERT INTO ExprTestTable VALUES {data} RETURNING ExprTestData
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let updated_row: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;
    assert_eq!(updated_row.str_field, "updated");
    Ok(())
})]
#[docify::export_content]
async fn update_struct_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let id = 1;
    let updated = ExprTestData {
        int_field: 5,
        str_field: "updated".to_string(),
        bool_field: false,
        nullable_field: None,
    };
    query!(&mut conn,
        UPDATE ExprTestTable SET {updated} WHERE id = {id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let updated_row: ExprTestData = query!(&mut conn,
        SELECT ExprTestData FROM ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;
    assert_eq!(updated_row.str_field, "inline");
    assert!(!updated_row.bool_field);
    Ok(())
})]
#[docify::export_content]
async fn update_inline_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let id = 1;
    query!(&mut conn,
        UPDATE ExprTestTable SET str_field = "inline", bool_field = false WHERE ExprTestTable.id = {id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(updated.len(), 2);
    assert!(updated.iter().all(|row| !row.bool_field));
    Ok(())
})]
#[docify::export_content]
async fn update_returning_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let bool_value = false;

    let updated: Vec<ExprTestData> = query!(&mut conn,
        UPDATE ExprTestTable SET bool_field = {bool_value}
        WHERE ExprTestTable.id > 1
        RETURNING Vec<ExprTestData>
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable WHERE ExprTestTable.id = 1
    )
    .await?;
    assert!(!exists);
    Ok(())
})]
#[docify::export_content]
async fn delete_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    query!(&mut conn, DELETE FROM ExprTestTable WHERE ExprTestTable.id = 1).await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert!(removed.is_some());
    Ok(())
})]
#[docify::export_content]
async fn delete_returning_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let removed: Option<ExprTestData> = query!(&mut conn,
        DELETE FROM ExprTestTable WHERE ExprTestTable.id = 1 RETURNING Option<ExprTestData>
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert!(exists);
    Ok(())
})]
#[docify::export_content]
async fn exists_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let exists: bool = query!(&mut conn,
        EXISTS ExprTestTable WHERE str_field IS NOT NULL And str_field = "exists"
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].data, "child");
    assert_eq!(rows[0].int_field, 5);
    Ok(())
})]
#[docify::export_content]
async fn table_joins_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    table_join!(ExprWithRelated | ExprTestTable INNER JOIN RelatedTestTable ON ExprTestTable.id = RelatedTestTable.parent_id);

    #[derive(Output)]
    #[sql(table = ExprWithRelated)]
    struct ExprRelatedOutput {
        #[sql(field = ExprTestTable.int_field)]
        int_field: i32,
        #[sql(field = RelatedTestTable.data)]
        data: String,
    }

    let parent_id = 1;
    let rows: Vec<ExprRelatedOutput> = query!(&mut conn,
        SELECT Vec<ExprRelatedOutput> FROM ExprWithRelated
        WHERE RelatedTestTable.parent_id = {parent_id}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    Ok(())
})]
#[docify::export_content]
async fn sql_function_builtin_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let rows: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable WHERE LOWER(str_field) = "hello"
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 1);
    Ok(())
})]
#[allow(non_local_definitions)]
#[docify::export_content]
async fn sql_function_custom_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    custom_sql_function!(Len; "LENGTH"; 1);
    let rows: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE Len(str_field) > 3
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(results.len(), 3);
    Ok(())
})]
#[docify::export_content]
async fn in_vec_example(mut conn: Transaction<'_, TestDriver>) -> anyhow::Result<()> {
    let ids = vec![10, 30, 50];
    let results: Vec<ExprTestData> = query!(&mut conn,
        SELECT Vec<ExprTestData> FROM ExprTestTable
        WHERE ExprTestTable.int_field IN {ids}
    )
    .await?;
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    assert_eq!(rows.len(), 2);
    Ok(())
})]
#[docify::export_content]
async fn generic_connection_example(
    conn: &mut &mut Transaction<'_, TestDriver>,
) -> anyhow::Result<()> {
    let rows: Vec<ExprTestData> =
        query!(*conn, SELECT Vec<ExprTestData> FROM ExprTestTable).await?;
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_query_basic_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<DocBasicTable>().await?;
    let mut conn = db.transaction().await?;

    let data = DocBasicData { column: 42 };
    query!(&mut conn, INSERT INTO DocBasicTable VALUES {data}).await?;

    query_basic_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_select_examples() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;
    select_examples(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_select_output_columns_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "low", false, None),
            expr_test_data(20, "mid", true, None),
            expr_test_data(30, "high", true, None),
        ],
    )
    .await?;

    select_output_columns_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_insert_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let conn = db.transaction().await?;

    insert_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_insert_returning_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let conn = db.transaction().await?;

    insert_returning_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_update_struct_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;
    update_struct_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_update_inline_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;
    update_inline_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_update_returning_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", true, None),
            expr_test_data(30, "c", false, None),
        ],
    )
    .await?;

    update_returning_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_delete_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;
    delete_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_delete_returning_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(&mut conn, default_expr_test_data()).await?;
    delete_returning_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_exists_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(
        &mut conn,
        ExprTestData {
            int_field: 1,
            str_field: "exists".to_string(),
            bool_field: true,
            nullable_field: None,
        },
    )
    .await?;

    exists_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_table_joins_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    RelatedTestTable::setup(&mut &mut conn).await?;

    let parent_data = ExprTestData {
        int_field: 5,
        str_field: "parent".to_string(),
        bool_field: true,
        nullable_field: None,
    };
    let _parent: ExprTestData = query!(&mut conn,
        INSERT INTO ExprTestTable VALUES {parent_data} RETURNING ExprTestData
    )
    .await?;

    let related_data = RelatedTestData {
        parent_id: 1,
        data: "child".to_string(),
    };
    query!(&mut conn, INSERT INTO RelatedTestTable VALUES {related_data}).await?;

    table_joins_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_sql_function_builtin_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(
        &mut conn,
        ExprTestData {
            int_field: 1,
            str_field: "Hello".to_string(),
            bool_field: true,
            nullable_field: None,
        },
    )
    .await?;

    sql_function_builtin_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_sql_function_custom_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_test_data(
        &mut conn,
        ExprTestData {
            int_field: 1,
            str_field: "abcd".to_string(),
            bool_field: true,
            nullable_field: None,
        },
    )
    .await?;

    sql_function_custom_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_in_vec_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(30, "b", true, None),
            expr_test_data(50, "c", false, None),
        ],
    )
    .await?;

    in_vec_example(conn).await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_generic_connection_example() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExprTestTable>().await?;
    let mut conn = db.transaction().await?;

    insert_multiple_test_data(
        &mut conn,
        vec![
            expr_test_data(10, "a", true, None),
            expr_test_data(20, "b", false, None),
        ],
    )
    .await?;

    let mut conn_ref = &mut conn;
    generic_connection_example(&mut conn_ref).await?;
    Ok(())
}
