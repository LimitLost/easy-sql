# Easy SQL

[![Crates.io](https://img.shields.io/crates/v/easy-sql.svg)](https://crates.io/crates/easy-sql)
[![Documentation](https://docs.rs/easy-sql/badge.svg)](https://docs.rs/easy-sql)
[![License](https://img.shields.io/crates/l/easy-sql.svg)](https://github.com/LimitLost/easy-sql/blob/master/LICENSE)

Easy SQL is a macro-first toolkit for writing SQL with strong compile-time guarantees. Based on [sqlx](https://crates.io/crates/sqlx)

- **Readable SQL** with IDE syntax highlighting (VS Code) inside [`query!`](#query-macros) and [`query_lazy!`](#query-macros).
- **Type-checked** column and table references, plus bind validation via `{value}`.
- **Clean bindings** embedded directly into the macro input, no separate `bind()` chain.
- **Optional migrations** see [#Migration system](#migration-system).
- **Optional table name checks** to prevent duplicates across files.
- **Interoperable with `sqlx`**: use `easy-sql` macros on SQLx connections/pools, or use migrations only.
- Currently supported drivers: **SQLite** and **Postgres**.

## Project Structure

- `-main/` — main `easy-sql` crate (library, drivers, tests, docs).
- `build/` — `easy-sql-build` helper crate for build-time setup and checks.
- `compilation-data/` — crate used for compile-time metadata.
- `macros/` — procedural macro crate powering derives and SQL macros.
- `scripts/` — helper scripts for tests and README tooling.

## Installation

Add the `easy-sql` dependency, then choose drivers and the matching `sqlx` runtime/TLS features.

### 1) Add `easy-sql`

Pick the driver features you need. Checking for duplicate table names is enabled by default. See [Feature flags](#feature-flags)

```toml
[dependencies]
easy-sql = { version = "0.101", features = ["sqlite", "postgres"] }
```

### 2) Add `sqlx` with runtime + TLS + driver

SQLx requires choosing **one runtime** and **optionally one TLS backend**, plus your database driver(s). From the official SQLx install guide:

- **Runtime**: `runtime-tokio` or `runtime-async-std`.
- **TLS**: (optional) `tls-native-tls` or one of the `tls-rustls-*` variants.
- **SQLite**: choose `sqlite` (bundled SQLite) or `sqlite-unbundled` (system SQLite).

Example using Tokio + Rustls + SQLite:

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "tls-rustls-ring-webpki", "sqlite"] }
```

See the full SQLx installation matrix at <https://github.com/launchbadge/sqlx#install>.

### 3) Add the build helper (optional but recommended)

The build helper crate is **`easy-sql-build`**.

Use it when you want:

- to not provide `#[sql(drivers = ...)]` for every Table by hand,
- specific driver compatibility checks
- [migrations](#migration-system),
- duplicate table name checks (`check_duplicate_table_names`),
- default drivers for [`query_lazy!`](#query-macros).

Add it as a build dependency:

```toml
[build-dependencies]
easy-sql-build = "1"
regex = "1" # needed only when you want to ignore certain files/directories in the build script
```

Example `build.rs`:

```rust
fn main() {
    easy_sql_build::build(
        // You can provide regex patterns to ignore certain files or directories
        &[regex::Regex::new(r"tests/.*").unwrap()],
        // Specify the locations of default drivers for Table setup generation and checks.
        &["easy_sql::Sqlite"],
    );
}
```

> ⚠️ **Do not** gitignore `easy_sql.ron`. It stores migration metadata + selected default drivers + list of all table names.

## Query macros

`query!` and `query_lazy!` give you readable SQL with compile-time validation and typed outputs.

- [`query!`](https://docs.rs/easy-sql/latest/easy_sql/macro.query.html) executes immediately and returns an `anyhow::Result<Output>`.
- [`query_lazy!`](https://docs.rs/easy-sql/latest/easy_sql/macro.query_lazy.html) builds a lazy query and returns a stream on execution.
  With `use_output_columns` feature, bare column references are referring to columns from the output type, instead of the table type.

## Mini demo

```rust,ignore
use easy_sql::sqlite::Database;
use easy_sql::{DatabaseSetup, Insert, Output, Table, query};

// DatabaseSetup lets you group tables into a single setup call.
#[derive(DatabaseSetup)]
struct PartOfDatabase {
    users: UserTable,
}

#[derive(Table)]
struct UserTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    email: String,
    active: bool,
}

#[derive(Insert)]
#[sql(table = UserTable)]
// Required to make sure that no fields are potentially ignored
#[sql(default = id)]
struct NewUser {
    email: String,
    active: bool,
}

#[derive(Output)]
#[sql(table = UserTable)]
struct UserRow {
    id: i32,
    #[sql(select = email || " (active = " || active || ")")]
    email_label: String,
    active: bool,
}
async fn main() -> anyhow::Result<()> {
    let db = Database::setup::<PartOfDatabase>("app.sqlite").await?;
    let mut conn = db.conn().await?;

    let data = NewUser {
        email: "sam@example.com".to_string(),
        active: true,
    };
    query!(&mut conn, INSERT INTO UserTable VALUES {data}).await?;

    let new_email = "sammy@example.com";
    query!(&mut conn,
        UPDATE UserTable SET active = false, email = {new_email} WHERE UserTable.email = "sam@example.com"
    )
    .await?;

    let row: UserRow = query!(&mut conn,
        SELECT UserRow FROM UserTable WHERE email = {new_email}
    )
    .await?;

    println!("{} {}", row.id, row.email_label);
    Ok(())
}
```

## Migration system

Migrations are optional and driven by table versions in [`Table`](https://docs.rs/easy-sql/latest/easy_sql/derive.Table.html) definitions. Use `migrations` feature to enable them.

1. **Create the table struct** and set `#[sql(version = 1)]`.
2. **Save/build** so the build helper can generate `#[sql(unique_id = "...")]` and register the version structure in `easy_sql.ron`.
3. **Update the table** (add/rename fields), then bump the version up.
4. **Save/build again** — the migration from version 1 is automatically generated and will be applied when (driver related) `Database::setup` or (table related) `DatabaseSetup::setup` are called.

Version tracking is stored in [`EasySqlTables`](https://docs.rs/easy-sql/latest/easy_sql/struct.EasySqlTables.html), and you can opt out with `#[sql(no_version)]` (needed only when `migrations` feature is enabled).

## Feature highlights (not everything)

- [`table_join!`](https://docs.rs/easy-sql/latest/easy_sql/macro.table_join.html) for typed joins.
- [`custom_sql_function!`](https://docs.rs/easy-sql/latest/easy_sql/macro.custom_sql_function.html) for custom SQL functions.
- `IN {vec}` binding with automatic placeholder expansion.
- `#[sql(select = ...)]` on [`Output`](https://docs.rs/easy-sql/latest/easy_sql/derive.Output.html) fields.
- `#[sql(bytes)]` for binary/serde storage.
- Composite primary keys and `#[sql(foreign_key = ...)]` relationships.

## Feature flags

Unless stated otherwise, feature is disabled by default.

- `sqlite`: Enable the SQLite driver.
- `postgres`: Enable the Postgres driver.
- `sqlite_math`: Enable extra SQLite math functions. Sqlite needs to be compiled with `LIBSQLITE3_FLAGS="-DSQLITE_ENABLE_MATH_FUNCTIONS"` for those functions to work.
- `migrations`: Enable migration generation and tracking.
- `check_duplicate_table_names` (default: ✅): Detect duplicate table names at build time.
- `use_output_columns`: Bare columns refer to output the type, instead of the table type.
- `bigdecimal`: Add `BigDecimal` `ToDefault` support (SQLx `bigdecimal`).
- `rust_decimal`: Add `Decimal` `ToDefault` support (SQLx `rust_decimal`).
- `uuid`: Add `Uuid` `ToDefault` support via SQLx.
- `chrono`: Add `chrono` `ToDefault` support via SQLx.
- `ipnet`: Add `ipnet` `ToDefault` support via SQLx.

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.
