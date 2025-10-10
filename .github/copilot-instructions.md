# GitHub Copilot Instructions for easy-sql

## Project Overview

This is a Rust crate that provides an ergonomic SQL ORM layer over SQLite/Postgres with proc macros for type-safe database operations. The architecture centers around compile-time code generation and build-time table validation.

## Critical Architecture Patterns

### Feature-Gated Library Structure

The crate uses conditional compilation extensively:

- `not_build` feature: Runtime SQL operations (default disabled)
- `build` feature: Build-time table generation and validation
- Database backends: `sqlite`, `postgres` features

When adding new functionality, ensure proper feature gating following the `src/lib.rs` pattern.

### Macro-Driven Development

Core functionality comes from proc macros in `sql-macros/`:

- `#[derive(SqlTable)]` - Generates table structs with versioning
- `#[derive(SqlInsert, SqlUpdate, SqlOutput)]` - CRUD operation structs
- `sql_where!()`, `sql_set!()` - Type-safe SQL clause generation
- `table_join!()` - Join table definitions

**Key Convention**: Always include `#[sql(version = N)]` on table structs for migration tracking.

### Build System Integration

The project uses `build.rs` with regex patterns to selectively process files:

```rust
sql_build::build(&[regex::Regex::new(r"example_all\.rs").unwrap()]);
```

This generates the `easy_sql.ron` file that tracks table schemas and versions for migrations.

### Database Connection Management

Connection patterns follow this hierarchy:

- `Database<DI>` - **Driver-specific** pool manager (e.g., `sqlite::Database<DI>` for SQLite)
- `Connection<D: Driver, DI>` - Generic single connection for queries
- `Transaction<'_, D: Driver, DI>` - Generic transactional wrapper

**Testing Convention**: Use `Database::setup_for_testing()` which auto-generates unique test database files and cleans them up.

## Essential Development Workflows

### Adding New Tables

1. Create struct with `#[derive(SqlTable)]` and `#[sql(version = 1)]`
2. Define fields with appropriate SQL attributes (`#[sql(primary_key)]`, `#[sql(foreign_key = OtherTable)]`)
3. Add to database setup struct with `#[derive(DatabaseSetup)]`
4. **Critical**: Never gitignore `easy_sql.ron` - it tracks schema evolution
5. Generated trait impls: One concrete implementation per selected driver (e.g., `impl SqlTable<Sqlite> for YourTable`, `impl SqlTable<Postgres> for YourTable`)

### CRUD Operations

Create separate structs for different operations:

```rust
#[derive(SqlInsert, SqlUpdate, SqlOutput)]
#[sql(table = YourTable)]
#[sql(default = auto_increment_field)]  // For SqlInsert
struct YourTableOps {
    field: String,
}
```

### Testing Patterns

- Tests go in `src/tests/` modules
- Use `#[sql_convenience]` attribute on async test functions
- Example: `src/tests/insert.rs`, `src/tests/select.rs`

### SQL Clause Construction

Use type-safe macros instead of raw SQL:

```rust
sql_where!(id = 5 AND name LIKE "%test%")
sql_set!(field = field + 1, name = "updated")
```

## Detailed Macro Syntax Patterns

### Table Definition Macros

**SqlTable Derive**:

```rust
#[derive(SqlTable)]
#[sql(version = 1)]                    // Required for migration tracking
#[sql(table_name = "custom_name")]     // Optional: override table name
struct ExampleTable {
    #[sql(primary_key)]                // Mark primary key fields
    #[sql(auto_increment)]             // For auto-incrementing IDs
    id: i32,

    #[sql(foreign_key = OtherTable)]   // Simple foreign key
    #[sql(foreign_key = OtherTable, cascade)]  // With cascade delete/update
    other_id: i32,

    #[sql(default = 42)]               // Column default values
    #[sql(default = "hello".to_string())]  // String defaults
    value: String,

    // Optional fields become nullable columns
    optional_field: Option<String>,
}
```

**Multi-Column Keys**:

```rust
#[derive(SqlTable)]
#[sql(version = 1)]
struct MultiKeyTable {
    #[sql(primary_key)]
    id1: i32,
    #[sql(primary_key)]
    id2: i64,

    // Multi-column foreign keys (order matters!)
    #[sql(foreign_key = MultiKeyTable, cascade)]
    ref_id1: i32,
    #[sql(foreign_key = MultiKeyTable)]  // Same table reference
    ref_id2: i64,
}
```

### CRUD Operation Macros

**SqlInsert/SqlUpdate/SqlOutput Patterns**:

```rust
#[derive(SqlInsert, SqlUpdate, SqlOutput)]
#[sql(table = ExampleTable)]
#[sql(default = id)]  // For SqlInsert: exclude auto-increment (or with for example default value) fields
struct ExampleOps {
    #[sql(field = custom_column_name)]  // Map to different column name
    rust_field: String,
    value: i32,
}

// Separate structs for different operations
#[derive(SqlOutput)]
#[sql(table = ExampleTable)]
struct ExampleSelect {
    id: i32,
    #[sql(field = ExampleTable.value)]  // Explicit table.column reference
    table_value: String,
}
```

### Query Construction Macros

**sql_where! Patterns**:

```rust
// Variable interpolation with curly braces
let user_id = 42;
sql_where!(id = {user_id} AND active = true)

// SQL operators and functions
sql_where!(name LIKE "%test%" OR age > 18)
sql_where!(created_at BETWEEN {start_date} AND {end_date})
sql_where!(status IN ("active", "pending"))

// Nested conditions
sql_where!((status = "active" OR status = "pending") AND age > {min_age})
```

**sql_set! Patterns**:

```rust
// Simple assignments
sql_set!(name = "new_name", age = 25)

// Arithmetic operations
sql_set!(counter = counter + 1, updated_at = {now})

// Function calls and expressions
sql_set!(total = price * quantity, status = UPPER({new_status}))
```

**table_join! Syntax**:

```rust
// Basic JOIN syntax
table_join!(UserPosts | User LEFT JOIN Post ON User.id = Post.user_id);

// Multiple JOINs
table_join!(ComplexQuery |
    User
    LEFT JOIN Post ON User.id = Post.user_id
    INNER JOIN Category ON Post.category_id = Category.id
);

// Use in SqlOutput
#[derive(SqlOutput)]
#[sql(table = UserPosts)]
struct UserPostOutput {
    #[sql(field = User.name)]
    user_name: String,
    #[sql(field = Post.title)]
    post_title: String,
}
```

### Testing and Convenience Macros

**sql_convenience Attribute**:

```rust
#[sql_convenience]  // Makes sql!, sql_where!, sql_set! macros take less input (related table information is provided automatically)
#[always_context]   // Automatically adds context to errors
async fn test_function() -> anyhow::Result<()> {
    // Test code with automatic database setup
    let db = Database::setup_for_testing::<TestDatabase>().await?;
    // ... test logic
    Ok(())
}
```

## Project-Specific Conventions

### Workspace Structure

- `sql-macros/` - Proc macro implementations
- `sql-build/` - Build-time code generation
- `sql-compilation-data/` - Shared types for build/runtime
- `src/drivers/` - Driver implementations
  - `src/drivers/sqlite/` - SQLite driver with value types and table operations
- Main crate exports conditionally based on features

### Error Handling

Uses `easy-macros` for context-aware error handling:

- `#[always_context]` on functions for automatic error context
- `anyhow::Result<T>` as standard return type

### Version Management

Table schemas are versioned in `easy_sql.ron`. When modifying tables:

1. Increment version number in `#[sql(version = N)]`
2. Never delete old versions from the RON file
3. Build system validates schema changes

### Foreign Key Patterns

Multi-column foreign keys must match order of referenced table:

```rust
#[sql(foreign_key = ReferencedTable, cascade)]  // First field gets cascade
field1: i32,
#[sql(foreign_key = ReferencedTable)]           // Subsequent fields reference same table
field2: i64,
```

## Integration Points

### SQLx Integration

- Uses `sqlx::Pool<D::InternalDriver>` for connection pooling
- Driver determines underlying sqlx database type
- Raw SQLx types accessible via: `SqlxRow`, `DriverRow<D>`
- Type aliases simplify common patterns: `DriverConnection<D>`, `DriverArguments<D>`

### Build Dependencies

- `easy-macros` - Error handling and proc macro utilities
- `sql-build` - Schema generation and validation
- Regex patterns control which files participate in schema generation

### Test Database Management

Test databases use auto-generated names and cleanup on drop. Never use production database paths in tests.

## Common Pitfalls

- **Lifetime Issues**: `SqlExpr` lifetime is invariant due to `D::Value<'a>` - use `unsafe transmute` with caution (see `update_set_clause.rs` and `sql_table.rs` for patterns)
- **Schema Evolution**: FIRST Modify table struct THEN increment version numbers. Always increment version numbers, never modify existing versions
- **Feature Gates**: Ensure runtime code is behind `not_build` feature
- **RON File**: Never ignore `easy_sql.ron` - it's essential for migrations
- **Build Regex**: Update build regex patterns when adding new schema-generating files
- **Connection Handling**: Use transactions for multi-statement operations, connections for single queries
