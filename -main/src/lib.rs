#![cfg_attr(docsrs, feature(doc_cfg))]

// Compile README.docify.md to README.md
#[cfg(feature = "_generate_readme")]
docify::compile_markdown!("README.docify.md", "README.md");

mod database_structs;
pub mod markers;
mod traits;

mod drivers;
pub use drivers::*;

pub use {
    database_structs::{Connection, EasySqlTables, PoolTransaction, Transaction},
    traits::{
        DatabaseSetup, Driver, EasyExecutor, EasyExecutorInto, Insert, Output, Table, ToDefault,
        Update,
    },
};
#[allow(rustdoc::broken_intra_doc_links)]
/// Driver-facing traits and helper types.
///
/// This module re-exports the low-level driver traits and capability markers used when
/// implementing a custom backend. Most application code will use the concrete drivers (like
/// [`Postgres`] or [`Sqlite`]) rather than these internals.
pub mod driver {
    pub use crate::database_structs::{AlterTable, AlterTableSingle, TableField};
    pub use crate::markers::driver::*;
    pub use crate::traits::{
        DriverArguments, DriverConnection, DriverQueryResult, DriverRow, DriverTypeInfo,
        InternalDriver, SetupSql,
    };
    /// Implement a built-in SQL function support marker for specific argument counts.
    ///
    /// This macro is intended for **custom drivers** to opt into the built-in SQL functions that
    /// [`query!`](crate::query)/[`query_lazy!`](crate::query_lazy) validate at compile time (e.g.
    /// `COUNT`, `SUBSTRING`, `DATE`). See [`supported`](crate::driver::supported) for which functions are available.
    ///
    /// ## Syntax
    /// ```rust,ignore
    /// impl_supports_fn!(DriverType, SupportsTrait, arg_count0, arg_count1, ...);
    /// ```
    ///
    /// - `SupportsTrait` comes from [`crate::markers::functions`] (for example [`SupportsCount`](crate::driver::SupportsCount)).
    /// - Provide one or more argument counts.
    /// - Use `-1` for functions that appear without parentheses (like `CURRENT_TIMESTAMP`).
    /// - Use [`impl_supports_fn_any`] for variadic functions.
    /// - For custom SQL functions (not built-ins), use [`custom_sql_function!`](crate::custom_sql_function).
    ///
    /// ## Example
    #[doc = docify::embed!(
		"src/tests/general/documentation/impl_supports_fn_macro.rs",
		impl_supports_fn_basic_example
	)]
    pub use easy_sql_macros::impl_supports_fn;
    /// Implement a built-in SQL function support marker for any argument count.
    ///
    /// Useful for variadic functions like `COALESCE` or `CONCAT`. This macro implements the
    /// corresponding marker trait for all `const ARGS: isize` counts, so
    /// [`query!`](crate::query)/[`query_lazy!`](crate::query_lazy) accept any number of arguments.  
    /// See [`supported`](crate::driver::supported) for which functions are available.
    ///
    /// ## Syntax
    /// ```rust,ignore
    /// impl_supports_fn_any!(DriverType, SupportsTrait);
    /// ```
    ///
    /// ## Example
    #[doc = docify::embed!(
		"src/tests/general/documentation/impl_supports_fn_macro.rs",
		impl_supports_fn_any_example
	)]
    pub use easy_sql_macros::impl_supports_fn_any;
}

pub mod supported;

#[cfg(test)]
mod tests;

#[doc(hidden)]
pub mod macro_support;

/// Type-safe SQL macro that builds and executes a query with compile-time checks.
///
/// Validates table/column names, binds arguments, and executes immediately, returns awaitable [anyhow](https://crates.io/crates/anyhow)::Result.
///
/// Notes:
/// - For INSERT/UPDATE/DELETE without `RETURNING`, the output is driver-specific (generally the
/// number of rows affected).
/// - Input syntax highlighting is applied in IDE's, but on documentation page it is not.
///
/// ## Syntax
/// ```rust,ignore
/// query!(<Driver> conn, SQL)
/// ```
///
/// - `<Driver>` is optional, if omitted the driver is inferred from `conn`.
/// - `conn` is generally a connection or transaction implementing [`EasyExecutor`], implemented for mutable versions of both, also implemented for Sqlx Executor types.
///
///  Example:
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", query_basic_example)]
///
/// - SQL keywords are case-insensitive.
/// - `{value}` inserts an external Rust value as a bound argument.
/// - See below for more info about the accepted SQL.
///
/// ## Query forms
///
/// ### SELECT
/// `SELECT OutputType FROM TableType ...` returns `OutputType`, a single row implementing [`Output`].
/// The `Output` trait also covers `Vec<T>` and `Option<T>` for multiple or optional rows.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", select_examples)]
///
/// Use `Table.column` or `OutputType.column` for column references. With the
/// `use_output_columns` feature, bare column names are validated against the output type instead of the Table type.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", select_output_columns_example)]
///
/// Accepted Clauses: `WHERE`, `GROUP BY`, `HAVING`, `ORDER BY`, `LIMIT`, `DISTINCT`
///
/// ### INSERT
/// `INSERT INTO TableType VALUES {data}` inserts one value or a collection. `{data}` must implement [`Insert`].
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", insert_example)]
///
/// Use `RETURNING OutputType` to return inserted data, OutputType needs to implement [`Output`].
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", insert_returning_example)]
///
///
/// ### UPDATE
/// `UPDATE TableType SET {update}` uses a struct implementing [Update], or
/// `SET field = value` for inline updates.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", update_struct_example)]
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", update_inline_example)]
///
/// Use `RETURNING OutputType` to return rows, OutputType needs to implement [`Output`].
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", update_returning_example)]
///
/// Accepted Clauses: `WHERE`, `RETURNING`
///
/// ### DELETE
/// `DELETE FROM TableType ...` removes rows.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", delete_example)]
///
/// Use `RETURNING OutputType` to return rows, OutputType needs to implement [`Output`]
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", delete_returning_example)]
///
/// Accepted Clauses: `WHERE`, `RETURNING`
///
/// ### EXISTS
/// `EXISTS TableType WHERE ...` returns `bool`.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", exists_example)]
///
/// Accepted Clauses: `WHERE`, `GROUP BY`, `HAVING`, `ORDER BY`, `LIMIT`
///
/// ### Table joins
/// Use [`table_join!`](crate::table_join) to define joins, then reference joined columns with
/// `JoinedTable.column`.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", table_joins_example)]
///
/// ### SQL functions
/// Built-in functions (like `COUNT`, `SUM`, `LOWER`) are available, and custom ones can be
/// registered with [`custom_sql_function!`](crate::custom_sql_function).
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", sql_function_builtin_example)]
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", sql_function_custom_example)]
///
/// ### `IN {vec}` parameter binding
/// `IN {vec}` expands placeholders at runtime. The value must implement `IntoIterator` and
/// `len()` (e.g., `Vec<T>`, `&[T]`). Use `IN {&vec}` if you need to reuse the collection.
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", in_vec_example)]
///
/// ## Generic connection
/// `*conn` syntax might be needed when using `&mut EasyExecutor<D>` as connection
#[doc = docify::embed!("src/tests/general/documentation/query_macro.rs", generic_connection_example)]
pub use easy_sql_macros::query;

/// Type-safe SQL macro that builds a lazy query and returns a stream on execution.
///
/// Like [`query!`], this macro validates table/column names and binds arguments at compile time,
/// but it does **not** execute immediately. Instead it returns a lazy query builder that can be
/// stored, moved, and executed later.
///
/// `?` (handling result) is required to build the lazy query.
///
/// ## Syntax
/// ```rust,ignore
/// query_lazy!(<Driver> SQL)
/// ```
///
/// - `<Driver>` is optional; if omitted the default driver is taken from the build script via
///   [`sql_build::build`](https://docs.rs/sql-build/latest/sql_build/fn.build.html).
///   If multiple (or zero) defaults are configured, you must specify the driver explicitly.
/// - Uses the same SQL syntax and helpers as [`query!`]
/// - There is **no connection parameter** in the macro call; pass it when fetching.
///
/// ### Return value
/// returns a `anyhow::Result<LazyQueryResult>` with the following method:
/// - `fetch(impl EasyExecutorInto)`
///     - when using a generic `&mut impl EasyExecutor` connection, use `fetch(&mut *conn)`
///     - otherwise pass a connection or transaction directly (e.g., `fetch(conn)` or `fetch(&mut transaction)`)
///
/// Both return `futures::Stream<Item = anyhow::Result<Output>>`. The stream borrows the
/// connection; drop or fully consume it before reusing the connection.
///
/// ### Output and query forms
/// - Output must be a **single-row type** implementing [`Output`]. To return multiple rows,
///   iterate/collect the stream yourself.
/// - Supported query forms: `SELECT`, `INSERT`, `UPDATE`, `DELETE`.
/// - `INSERT`/`UPDATE`/`DELETE` **must** include `RETURNING` (because results are streamed).
/// - `EXISTS` is **not** supported; use [`query!`] instead.
///
/// ## Examples
#[doc = docify::embed!("src/tests/general/documentation/query_lazy_macro.rs", query_lazy_basic_example)]
#[doc = docify::embed!("src/tests/general/documentation/query_lazy_macro.rs", query_lazy_streaming_example)]
#[doc = docify::embed!("src/tests/general/documentation/query_lazy_macro.rs", generic_executor_example)]
pub use easy_sql_macros::query_lazy;

/// Defines a SQL table schema.
///
/// Implements [`Table`], [`DatabaseSetup`], [`Output`], [`Insert`], and [`Update`] for the struct,
/// for usage inside of `query!` macros. Table names default to the struct
/// name converted to `snake_case`.
///
/// ## Basic usage
#[doc = docify::embed!("src/tests/general/documentation/table_macro.rs", table_basic_example)]
///
/// ## Field attributes
/// - `#[sql(primary_key)]` marks a column as part of the primary key.
/// - `#[sql(auto_increment)]` enables auto-increment for the column (driver-dependent).
/// - `#[sql(unique)]` adds a `UNIQUE` constraint.
/// - `#[sql(default = expr)]` sets a column default (the expression is type-checked).
/// - `#[sql(bytes)]` stores the field as a binary blob using [`bincode`](https://crates.io/crates/bincode) + [`serde`](https://crates.io/crates/serde).
/// - `#[sql(foreign_key = TableStruct)]` creates a foreign key to another table.
/// - `#[sql(foreign_key = TableStruct, cascade)]` enables `ON DELETE/UPDATE CASCADE`.
///
/// `Option<T>` fields are treated as nullable; all other fields are `NOT NULL` by default.
///
/// ### Foreign keys
#[doc = docify::embed!("src/tests/general/documentation/table_macro.rs", table_foreign_key_example)]
///
/// ### Composite primary keys
#[doc = docify::embed!(
	"src/tests/general/documentation/table_macro.rs",
	table_composite_primary_key_example
)]
///
/// ## Table attributes
/// - `#[sql(table_name = "...")]` overrides the generated `snake_case` name.
/// - `#[sql(drivers = Driver1, Driver2)]` sets the supported drivers when no default driver is
///   configured in the build script via
///   [`sql_build::build`](https://docs.rs/sql-build/latest/sql_build/fn.build.html).
/// - `#[sql(no_version)]` disables migrations for this table (feature `migrations`).
/// - `#[sql(version = 1)]` enables migrations and sets the table version (feature `migrations`).
/// - `#[sql(unique_id = "...")]` is auto generated and used by the build script for migration tracking.
/// - `#[sql(version_test = 1)]` sets a temporary version for migration testing, requires `unique_id`.
///
/// ## Notes
/// - Some drivers require at least one primary key; if none is specified, compilation will fail.
/// - Auto-increment may be restricted when using composite primary keys, depending on the driver.
/// - `#[sql(bytes)]` requires the field type to implement [`serde::Serialize`](https://docs.rs/serde/latest/serde/trait.Serialize.html)/[`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html).
pub use easy_sql_macros::Table;

/// Defines insertable data for a table.
///
/// Implements [`Insert`] for the `Struct` and `&Struct`, for use in [`query!`](crate::query) and [`query_lazy!`](crate::query_lazy).
/// Field names map to table columns, and you can provide a subset of columns as long as the
/// missing ones are declared as defaults.
///
/// ## Basic usage
#[doc = docify::embed!("src/tests/general/documentation/insert_macro.rs", insert_basic_example)]
///
/// ## Defaults for omitted columns
/// Use `#[sql(default = field1, field2)]` to declare table columns that are **not** present in the
/// insert struct (for example, auto-increment primary keys or columns with SQL defaults). All
/// omitted columns must be listed so the macro can validate the table schema at compile time.
#[doc = docify::embed!("src/tests/general/documentation/insert_macro.rs", insert_defaults_example)]
///
/// ## Field attributes
/// - `#[sql(bytes)]` must match `#[sql(bytes)]` on table struct, stores the field as a binary blob using [`bincode`](https://crates.io/crates/bincode) + [`serde`](https://crates.io/crates/serde).
///
/// ## Notes
/// - `#[sql(table = TableStruct)]` is required and must point to a [`Table`] type.
/// - You can insert a single value, borrow or a collection (`&T`, `Vec<T>`, `&Vec<T>`, `&[T]`).
pub use easy_sql_macros::Insert;
/// Defines a SQL output mapping.
///
/// Implements [`Output`] for the struct, enabling type-safe selection and decoding of rows in
/// [`query!`] and [`query_lazy!`]. Column names are validated at compile time against the table
/// or joined tables.
///
/// ## Basic usage
#[doc = docify::embed!("src/tests/general/documentation/output_macro.rs", output_basic_example)]
///
/// ## Custom select expressions
/// Use `#[sql(select = ...)]` to map a field to a SQL expression. The expression is emitted into
/// the `SELECT` list with an alias matching the field name.
#[doc = docify::embed!("src/tests/general/documentation/output_macro.rs", output_custom_select_example)]
///
/// ### Custom select arguments
/// Use `{arg0}`, `{arg1}`, ... inside `#[sql(select = ...)]` and pass arguments in the query as
/// `SELECT OutputType(arg0, arg1) ...` or `RETURNING OutputType(arg0, arg1)`.
#[doc = docify::embed!("src/tests/general/documentation/output_macro.rs", output_custom_select_args_example)]
///
/// ## Joined output fields
/// For [`table_join!`](crate::table_join) results, map fields to joined columns with `#[sql(field = Table.column)]` or `#[sql(select = ...)]`.
#[doc = docify::embed!("src/tests/general/documentation/output_macro.rs", output_joined_fields_example)]
///
/// ## Field attributes
/// - `#[sql(select = ...)]` maps the field to a custom SQL expression.
/// - `#[sql(field = Table.column)]` maps the field to a joined table column.
/// - `#[sql(bytes)]` decodes the field from binary data (matches table field settings).
///
/// ## Table attributes
/// - `#[sql(table = TableStruct)]` is required and sets the base table for validation.
///
/// ## Notes
/// - Output structs represent a **single row**; wrap in `Vec<T>` or `Option<T>` for multiple or
///   optional rows.
/// - With `use_output_columns`, you can reference `OutputType.column` (or bare columns) in
///   `query!` expressions.
/// - Custom select arguments must be sequential (`arg0`, `arg1`, ...).
pub use easy_sql_macros::Output;

/// Defines update data for a table.
///
/// Implements [`Update`] for the struct and `&Struct`, for use in [`query!`](crate::query) and
/// [`query_lazy!`](crate::query_lazy). Field names map to table columns, and only the fields you
/// define are updated. `Option<T>` values are bound as-is, so `None` sets the column to `NULL`.
///
/// ## Basic usage
#[doc = docify::embed!("src/tests/general/documentation/update_macro.rs", update_basic_example)]
///
/// ## Partial updates
/// Define a smaller update struct to update a subset of columns. Use `&data` if you need to reuse
/// the update payload for multiple queries.
#[doc = docify::embed!("src/tests/general/documentation/update_macro.rs", update_partial_example)]
///
/// ## Field attributes
/// - `#[sql(bytes)]` must match `#[sql(bytes)]` on the table struct, stores the field as a binary
///   blob using [`bincode`](https://crates.io/crates/bincode) + [`serde`](https://crates.io/crates/serde).
/// - `#[sql(maybe_update)]` / `#[sql(maybe)]` marks an `Option<T>` field as optional: `None` skips the update while
///   `Some(value)` updates the column. For nullable columns you can also use `Option<Option<T>>`
///   to allow `Some(None)` to set `NULL`.
///
/// ## Notes
/// - `#[sql(table = TableStruct)]` is required and must point to a [`Table`] type.
pub use easy_sql_macros::Update;

/// Composes database structure from nested types.
///
/// Implements [`DatabaseSetup`] by calling `setup` on each field (in order), which makes it easy
/// to group tables or other setup structs into reusable schemas.
///
/// - Works with named **or tuple** structs.
/// - Use `#[sql(drivers = Driver1, Driver2)]` to select supported drivers when no defaults are
///   configured in the build script via
///   [`sql_build::build`](https://docs.rs/sql-build/latest/sql_build/fn.build.html).
/// - Errors include the field name and type to help trace failing setup calls.
///
/// ## Basic usage
#[doc = docify::embed!(
	"src/tests/general/documentation/database_setup_macro.rs",
	database_setup_basic_example
)]
///
/// ## Nested setup groups
#[doc = docify::embed!(
	"src/tests/general/documentation/database_setup_macro.rs",
	database_setup_nested_example
)]
///
/// ## Notes
/// - Every field must implement [`DatabaseSetup`] for the selected driver(s).
/// - Setup order follows field order;
pub use easy_sql_macros::DatabaseSetup;

/// Defines a joined table type for use in [`query!`](crate::query) and [`query_lazy!`](crate::query_lazy).
///
/// `table_join!` creates a lightweight type that implements [`Table`] with a generated join clause.
/// Use it as the table name in queries and as the `#[sql(table = ...)]` target for [`Output`].
///
/// ## Syntax
/// ```rust,ignore
/// table_join!(<Driver1, Driver2> JoinedTableStructName | TableA INNER JOIN TableB ON TableA.id = TableB.a_id)
/// ```
///
/// - The driver list is optional; when omitted, default drivers from the build script via
///   [`sql_build::build`](https://docs.rs/sql-build/latest/sql_build/fn.build.html) are used.
/// - Supported join types: `INNER JOIN`, `LEFT JOIN`, `RIGHT JOIN`, `CROSS JOIN`.
/// - `CROSS JOIN` omits the `ON` clause.
/// - Multiple joins can be chained after the main table.
///
/// ## Output mapping
/// - Use `#[sql(field = Table.column)]` or `#[sql(select = ...)]` in [`Output`] structs.
/// - `LEFT JOIN` makes the joined table optional; map its fields as `Option<T>`.
/// - `RIGHT JOIN` makes tables to the **left** optional; map their fields as `Option<T>`.
///
/// ## Examples
#[doc = docify::embed!(
	"src/tests/general/documentation/table_join_macro.rs",
	table_join_basic_example
)]
#[doc = docify::embed!(
	"src/tests/general/documentation/table_join_macro.rs",
	table_join_left_example
)]
pub use easy_sql_macros::table_join;

/// Define a custom SQL function for use in [`query!`](crate::query) and [`query_lazy!`](crate::query_lazy).
///
/// Registers a SQL function name and argument-count validation so the query macros can parse
/// calls like `MyFunc(column, "arg")` and enforce argument counts at compile time.
///
/// ## Syntax
/// ```rust,ignore
/// custom_sql_function!(FunctionName; "SQL_FUNCTION_NAME"; 0 | 1 | 2 | Any);
/// ```
///
/// - `FunctionName`: Rust identifier used to generate the macro name.
/// - `"SQL_FUNCTION_NAME"`: SQL function name emitted in generated SQL.
/// - `args`: Argument count specification:
///   - A number (e.g., `2`) for exact argument count
///   - Multiple numbers separated by `|` (e.g., `1 | 2`) for multiple allowed counts
///   - `Any` for any number of arguments
///
/// ## Examples
#[doc = docify::embed!(
	"src/tests/general/documentation/custom_sql_function_macro.rs",
	custom_sql_function_basic_example
)]
#[doc = docify::embed!(
	"src/tests/general/documentation/custom_sql_function_macro.rs",
	custom_sql_function_multiple_args_example
)]
#[doc = docify::embed!(
	"src/tests/general/documentation/custom_sql_function_macro.rs",
	custom_sql_function_any_args_example
)]
///
/// ## Notes
/// - Function names are case-insensitive in queries, but the emitted SQL name preserves casing.
/// - The macro only validates SQL syntax; the function must exist in your database.
/// - `Any` disables argument-count validation; otherwise invalid counts cause a compile-time error.
pub use easy_sql_macros::custom_sql_function;
