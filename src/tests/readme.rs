mod easy_lib {
    pub use crate as sql;
}

use easy_lib::sql::{DatabaseSetup, SqlTable};

#[derive(SqlTable)]
// Needed because of automatic migration generation
// Update this after you're done with making changes (NOT before)
#[sql(version = 1)]
#[sql(unique_id = "c41fcf81-08e2-4b89-98e7-57dce7d984d6")]
struct ExampleTable {
    // Column name: `id`
    // Multiple primary keys supported
    #[sql(primary_key)]
    id: i32,
    // Column name: `field1`
    field1: String,
}

//Sub database, doesn't change anything in internal Sqlite database
#[derive(DatabaseSetup)]
struct ExampleSubDatabase {
    t1: ExampleTable,
    t2: ExampleTableIncrement,
}

#[derive(DatabaseSetup)]
struct ExampleDatabase {
    sub: ExampleSubDatabase,
}

use easy_lib::sql::sqlite::Database;

// Save connection pool in a global variable
// Use `lazy_static` library and `std::sync::Mutex` to do that
lazy_static::lazy_static! {
   static ref DB_BASE: std::sync::Mutex<Option<Database>> = Default::default();
   static ref DB: Database = DB_BASE.lock().unwrap().take().unwrap();
}

//Connect to database and save it for later use
#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let db = Database::setup::<ExampleDatabase>("example.db", Default::default()).await?;
    *DB_BASE.lock().unwrap() = Some(db);
    Ok(())
}

#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(unique_id = "21d36640-7002-49d4-b373-3a2d17c61ff1")]
struct ExampleTableIncrement {
    // Column name: `id`
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    // Column name: `field`
    field: i64,
}

#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(unique_id = "9a5884dd-3f0c-4323-bd5f-07fd1bbb10ed")]
struct ExampleTableMultiPrimaryKey {
    #[sql(primary_key)]
    id: i32,
    #[sql(primary_key)]
    id2: i64,
}

#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(unique_id = "822ee817-0738-441e-b222-29a235fa7be3")]
struct ExampleTableWithForeignKey {
    #[sql(primary_key)]
    id: i32,
    /// Without cascade on update/delete
    #[sql(foreign_key = ExampleTableIncrement)]
    example_increment_id: i32,
    /// With cascade on update/delete
    #[sql(foreign_key = ExampleTable, cascade)]
    example_table_id: i32,
    value: String,
}

#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(unique_id = "2f9ce3ef-1506-4884-985c-6ebfd4a0c54c")]
struct ExampleTableWithMultiForeignKey {
    #[sql(primary_key)]
    id: i32,
    // Columns must be in the same order as in the foreign table
    // Only one of fields needs the "cascade" attribute (if you want it)
    #[sql(foreign_key = ExampleTableMultiPrimaryKey, cascade)]
    example_table_id: i32,
    #[sql(foreign_key = ExampleTableMultiPrimaryKey)]
    example_table_id2: i64,
}

#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(unique_id = "5abb2707-0b7c-486c-8ca4-beac5f4af281")]
struct ExampleTableDefaultValue {
    #[sql(primary_key)]
    id: i32,
    #[sql(default = 5)]
    field: i64,
    #[sql(default = "Hello world".to_string())]
    field_str: String,
}

#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(table_name = "example_table_renamed_to_something_else")]
#[sql(unique_id = "9d9e3d2d-f9dc-406c-96ff-e9a33c9da0c1")]
struct ExampleTableRenamed {
    #[sql(primary_key)]
    id: i32,
    field: i64,
}

use easy_lib::sql::{SqlInsert, SqlOutput, SqlUpdate};

#[derive(SqlInsert, SqlUpdate, SqlOutput)]
#[sql(table = ExampleTableIncrement)]
#[sql(default = id)]
struct ExampleInsert {
    pub field: i64,
}

use easy_lib::sql::{sql, sql_convenience, sql_set, sql_where};

#[tokio::test]
#[sql_convenience]
async fn main2() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<ExampleDatabase>().await?;

    // You can also use `db.conn()` if you don't want to start a transaction
    let mut conn = db.transaction().await?;

    // TODO Every (returning) function below has lazy variant, for example `insert_returning_lazy` and `get_lazy`
    // Which loads one row at a time, instead of all rows at once

    // Inserting data
    // There's also `insert_returning`
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert { field: 5 }).await?;
    //sql_where! macro uses SQLite syntax
    // There's also `update_returning`
    ExampleTableIncrement::update(&mut conn, &ExampleInsert { field: 10 }, sql_where!(id = 3))
        .await?;

    //Use sql_set! macro to write more complex value updates
    ExampleTableIncrement::update(
        &mut conn,
        sql_set!(field = field + 5, id = id * 1),
        sql_where!(id = 3),
    )
    .await?;

    // Selecting data
    let example_result_vec: Vec<ExampleInsert> =
        ExampleTableIncrement::select(&mut conn, sql_where!(id = 2)).await?;
    // Get is an alias for select
    let example_result_single: ExampleInsert =
        ExampleTableIncrement::get(&mut conn, sql_where!(id = 1)).await?;

    // sql! macro allows to do more complex queries
    let example_result_maybe_single: Option<ExampleInsert> =
        ExampleTableIncrement::select(&mut conn, sql!(WHERE id = 2 ORDER BY id)).await?;

    // There's also `delete_returning`
    ExampleTableIncrement::delete(&mut conn, sql_where!(id = 1)).await?;

    // You can also use `conn.rollback().await?` if you want to rollback the transaction
    conn.commit().await?;
    Ok(())
}

use easy_lib::sql::table_join;
use sql_macros::table_join_debug;

table_join!(JoinedExampleTables | ExampleTable LEFT JOIN ExampleTableWithForeignKey ON ExampleTable.id = ExampleTableWithForeignKey.example_table_id);

#[derive(SqlOutput)]
#[sql(table = JoinedExampleTables)]
struct JoinedExampleTableOutput {
    //You need to specify referenced table column
    #[sql(field = ExampleTable.id)]
    id: i32,
    #[sql(field = ExampleTableWithForeignKey.value)]
    value: Option<String>,
}
