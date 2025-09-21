Currently this library only supports SQLite.

# Future Features

- Table join support
- Allow for multiple `#[sql(table = ...)]` attributes on single struct
- Support for Postgres
- Support for syncing data to remote database provided by you
- Renaming columns in table (with attribute, overwriting name set by the field name)
- (Table editing) Support for changing more than just renaming table, columns and adding new columns
- Check if foreign key column type is correct

# Examples (they reference each other)

- `.gitignore` setup (ignore build logs)

```gitignore
/easy_sql_logs
```

WARNING: Never gitignore `easy_sql.ron` file, it is used for generating migrations (and for checking foreign keys in the future)

## Creating database and tables

- define database structure and a simple table

```rust
use easy_lib::sql::{DatabaseSetup, SqlTable};

#[derive(SqlTable)]
// Needed because of automatic migration generation
// Update this after you're done with making changes (NOT before)
#[sql(version = 1)]
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
}

#[derive(DatabaseSetup)]
struct ExampleDatabase {
   sub: ExampleSubDatabase,
}
```

- execute database creation

```rust
use easy_lib::sql::Database;

// Save connection pool in a global variable
// Use `lazy_static` library and `std::sync::Mutex` to do that
lazy_static::lazy_static! {
   static ref DB_BASE: std::sync::Mutex<Option<Database>> = Mutex::new(None);
   static ref DB: Database = DB_BASE.lock().unwrap().take().unwrap();
}

//Connect to database and save it for later use
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Database::setup::<ExampleDatabase>("example.db").await?;
    *DB_BASE.lock().unwrap() = Some(db);
    Ok(())
}
```

## Advanced table creation

- Primary Key Auto Increment

```rust
#[derive(SqlTable)]
#[sql(version = 1)]
struct ExampleTableIncrement {
    // Column name: `id`
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    // Column name: `field`
    field: i64,
}
```

- Multi column primary key

```rust
#[derive(SqlTable)]
#[sql(version = 1)]
struct ExampleTableMultiPrimaryKey {
    #[sql(primary_key)]
    id: i32,
    #[sql(primary_key)]
    id2: i64,
}
```

- with Foreign Key

```rust

#[derive(SqlTable)]
#[sql(version = 1)]
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

```

- Multi column foreign key

```rust
#[derive(SqlTable)]
#[sql(version = 1)]
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
```

- Default value for columns

```rust
#[derive(SqlTable)]
#[sql(version = 1)]
struct ExampleTableDefaultValue {
    #[sql(primary_key)]
    id: i32,
    #[sql(default = 5)]
    field: i64,
    #[sql(default = "Hello world".to_string())]
    field_str: String,
}
```

- Table Renaming

```rust
#[derive(SqlTable)]
#[sql(version = 1)]
#[sql(table_name = "example_table_renamed_to_something_else")]
struct ExampleTableRenamed {
    #[sql(primary_key)]
    id: i32,
    field: i64,
}
```

## Table manipulation

- Creating table manipulation structs with `SqlInsert`, `SqlUpdate` and `SqlOutput` derive macros

```rust
use easy_lib::sql::{SqlInsert, SqlUpdate, SqlOutput};

//Field validity is automatically checked and errors will be shown on compile time if they are not
#[derive(SqlInsert,SqlUpdate,SqlOutput)]
#[sql(table = ExampleTableIncrement)]
// because of `SqlInsert` you need to specify which table columns are default (not specified in the insert statement)
// Multiple column example: #[sql(default = id, field2)]
#[sql(default = id)]
struct ExampleInsert{
    field: i64,
}

```

- Table manipulation functions

```rust
use easy_lib::sql::{sql_where,sql,sql_set,sql_convenience};

#[tokio::main]
#[sql_convenience]
async fn main2() -> anyhow::Result<()> {
    let db = Database::setup::<ExampleDatabase>("example.db").await?;

    // You can also use `db.conn()` if you don't want to start a transaction
    let mut conn=db.transaction().await?;
`
    // TODO Every (returning) function below has lazy variant, for example `insert_returning_lazy` and `get_lazy`
    // Which loads one row at a time, instead of all rows at once

    // Inserting data
    // There's also `insert_returning`
    ExampleTableIncrement::insert(&mut conn, &ExampleInsert{field: 5}).await?;

    //sql_where! macro uses SQLite syntax
    // There's also `update_returning`
    ExampleTableIncrement::update(&mut conn, ExampleInsert{field: 10}, sql_where!(id = 3)).await?;
    //Use sql_set! macro to write more complex value updates
    ExampleTableIncrement::update(
        &mut conn,
        sql_set!(field = field + 5, id = id * 1),
        sql_where!(id = 3),
    )
    .await?;

    // There's also `delete_returning`
    ExampleTableIncrement::delete(&mut conn, sql_where!(id = 1)).await?;

    // Selecting data
    let example_result_vec:Vec<ExampleInsert> = ExampleTableIncrement::select(&mut conn, sql_where!(id = 2)).await?;
    // Get is an alias for select
    let example_result_single:ExampleInsert = ExampleTableIncrement::get(&mut conn, sql_where!(id = 2)).await?;
    // sql! macro allows to do more complex queries
    let example_result_maybe_single:Option<ExampleInsert> = ExampleTableIncrement::select(&mut conn, sql!(WHERE id = 2 ORDER BY id)).await?;

    // You can also use `conn.rollback().await?` if you want to rollback the transaction
    conn.commit().await?;
    Ok(())
}
```

## Table Joining

- Creating joined table struct

```rust
use easy_lib::sql::table_join;

// First Argument - Struct Name Representing the Joined Tables
// `|` - Separator
table_join!(JoinedExampleTables | ExampleTable LEFT JOIN ExampleTableWithForeignKey ON ExampleTable.id = ExampleTableWithForeignKey.example_table_id);

```

- Creating joined table data output

```rust
#[derive(SqlOutput)]
#[sql(table = JoinedExampleTables)]
struct JoinedExampleTableOutput {
    //You need to specify referenced table column
    #[sql(field = ExampleTable.id)]
    id: i32,
    #[sql(field = ExampleTableWithForeignKey.value)]
    value: String,
}
```

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
