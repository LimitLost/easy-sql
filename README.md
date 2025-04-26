Currently this library only supports SQLite.

# Future Features

- Support for Postgres (connection made by external handler provided by you)
- Renaming columns in table (with attribute, overwriting name set by the field name)

# Examples (they reference each other)

## Creating database and tables

- define database structure and a simple table

```rust
use easy_lib::sql::{DatabaseSetup, SqlTable};

#[derive(SqlTable)]
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

## Table manipulation
