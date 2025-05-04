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

- TODO Multi column foreign key

- TODO show Table Renaming

## Table manipulation

- Creating table manipulation structs with `SqlInsert`, `SqlUpdate` and `SqlOutput` derive macros

```rust

//Field validity is automatically checked and errors will be shown on compile time if they are not
#[derive(SqlInsert,SqlUpdate,SqlOutput)]
#[sql(table = ExampleTableIncrement)]
struct ExampleInsert{
    field: i64,
}

```

- TODO Create example using all table manipulation functions

```rust
use easy_lib::sql::{SqlTable};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = Database::setup::<ExampleDatabase>("example.db").await?;

    // You can also use `db.conn()` if you don't want to start a transaction
    let mut conn=db.transaction().await?;
`
    // TODO Your code for table manipulation will go here

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
