Currently this library only supports SQLite.

# Future Features

- Table join support
- EXISTS query support
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

- TODO show auto increment

- TODO show multiple primary keys

- TODO show foreign keys

- TODO show table renaming

## Table manipulation

- TODO Create example using all table manipulation functions
