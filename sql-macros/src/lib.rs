mod macros;
mod macros_components;
mod query_macro_components;
mod sql_convenience_attr;

mod derive;

use ::{proc_macro2, quote::quote};
use easy_macros::{always_context, anyhow_result, find_crate_list};
use proc_macro::TokenStream;

fn sql_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[("easy-sql", quote! {})]) {
        found
    } else {
        quote! {self}
    }
}

/// Type-safe SQL macro supporting three modes: expressions, SET clauses, and SELECT clauses.
///
/// # Modes
///
/// ## 1. SQL Expressions (WHERE conditions)
/// ```rust
/// // Simple conditions
/// Table::get(&mut conn, sql!(id = 1)).await?;
/// Table::select(&mut conn, sql!(status = "active")).await?;
///
/// // Complex conditions with AND/OR
/// Table::select(&mut conn, sql!(id >= 2 AND field <= 40)).await?;
/// Table::select(&mut conn, sql!(name LIKE "test%")).await?;
///
/// // Variable interpolation with {}
/// let user_id = 42;
/// Table::get(&mut conn, sql!(id = {user_id})).await?;
/// Table::select(&mut conn, sql!(field = {i64::MAX})).await?;
///
/// // Multiple variables
/// Table::select(&mut conn, sql!(age > {min_age} AND status = {status_var})).await?;
/// ```
///
/// ## 2. SET Clauses (UPDATE operations)
/// The `SET` keyword is automatically added by `#[sql_convenience]` or can be added manually.
/// ```rust
/// // Multiple field updates (comma-separated)
/// Table::update(&mut conn, sql!(field = 99, name = "new"), sql!(id = 1)).await?;
///
/// // With variable interpolation
/// let new_value = 100;
/// Table::update(&mut conn, sql!(counter = {new_value}), sql!(id = 1)).await?;
///
/// // Arithmetic operations
/// Table::update(&mut conn, sql!(counter = counter + 1), sql!(id = 1)).await?;
///
/// // Or manually specify SET keyword
/// Table::update(&mut conn, sql!(SET field = field * 2), sql!(id = 1)).await?;
/// ```
///
/// ## 3. SELECT Clauses (full query clauses)
/// ```rust
/// // WHERE with variable interpolation
/// let min_id = 5;
/// Table::select(&mut conn, sql!(WHERE id > {min_id})).await?;
///
/// // ORDER BY
/// Table::select(&mut conn, sql!(ORDER BY field ASC)).await?;
///
/// // LIMIT
/// Table::select(&mut conn, sql!(LIMIT 10)).await?;
///
/// // Combined clauses
/// Table::select(&mut conn, sql!(WHERE field >= 30 ORDER BY field DESC LIMIT 3)).await?;
///
/// // DISTINCT
/// Table::select(&mut conn, sql!(DISTINCT WHERE category = "active")).await?;
///
/// // BETWEEN with variables
/// Table::select(&mut conn, sql!(WHERE created_at BETWEEN {start_date} AND {end_date})).await?;
/// ```
///
/// # Mode Detection
/// - **Expression**: No SQL keywords, single expression (e.g., `id = 1`)
/// - **SET Clause**: Starts with `SET` keyword (added by hand or automatically) (e.g., `SET field = value` or `field = value, name = "x"`)
/// - **SELECT Clauses**: Starts with `WHERE`, `ORDER`, `GROUP`, `HAVING`, `LIMIT`, or `DISTINCT`
///
/// # Type Safety
/// All field names are validated at compile-time against the table schema when used with `#[sql_convenience]` or when user add table struct info manually example: `sql!(|Table| id = 1)`.
#[proc_macro]
#[always_context]
#[anyhow_result]
pub fn sql(item: TokenStream) -> anyhow::Result<TokenStream> {
    macros::sql(item)
}

#[always_context]
/// Debug version of `sql!` that prints the generated code and panics.
/// Useful for inspecting macro expansion during development.
#[proc_macro]
#[anyhow_result]
#[no_context]
pub fn sql_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = macros::sql(item)?;
    panic!("{}", result);
}

/// Type-safe query! macro for executing SQL queries.
///
/// # Usage
///
/// ## SELECT Query
/// ```rust
/// // Basic SELECT
/// let result = query!(&mut conn, SELECT OutputType FROM TableType WHERE id = {user_id}).await?;
///
/// // With ORDER BY and LIMIT
/// let results = query!(&mut conn, SELECT OutputType FROM TableType WHERE status = "active" ORDER BY created_at DESC LIMIT 10).await?;
/// ```
///
/// ## INSERT Query
/// ```rust
/// // INSERT without RETURNING
/// query!(&mut conn, INSERT INTO TableType VALUES {data}).await?;
///
/// // INSERT with RETURNING
/// let inserted = query!(&mut conn, INSERT INTO TableType VALUES {data} RETURNING OutputType).await?;
/// ```
///
/// ## UPDATE Query
/// ```rust
/// // UPDATE without RETURNING
/// query!(&mut conn, UPDATE TableType SET field = {new_value} WHERE id = {user_id}).await?;
///
/// // UPDATE with RETURNING
/// let updated = query!(&mut conn, UPDATE TableType SET field = {new_value} WHERE id = {user_id} RETURNING OutputType).await?;
/// ```
///
/// ## DELETE Query
/// ```rust
/// // DELETE without RETURNING
/// query!(&mut conn, DELETE FROM TableType WHERE id = {user_id}).await?;
///
/// // DELETE with RETURNING
/// let deleted = query!(&mut conn, DELETE FROM TableType WHERE id = {user_id} RETURNING OutputType).await?;
/// ```
///
/// ## EXISTS Query
/// ```rust
/// // Check if records exist
/// let exists: bool = query!(&mut conn, EXISTS TableType WHERE email = {user_email}).await?;
/// ```
///
/// # Type Safety
/// All field names and types are validated at compile-time. The macro generates proper
/// parameter binding and SQL construction code.
#[proc_macro]
#[always_context]
#[anyhow_result]
pub fn query(item: TokenStream) -> anyhow::Result<TokenStream> {
    macros::query(item)
}

#[always_context]
/// Debug version of `query!` that prints the generated code and panics.
/// Useful for inspecting macro expansion during development.
#[proc_macro]
#[anyhow_result]
#[no_context]
pub fn query_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = macros::query(item)?;
    panic!("{}", result);
}

#[always_context]
#[proc_macro]
#[anyhow_result]
pub fn table_join(item: TokenStream) -> anyhow::Result<TokenStream> {
    macros::table_join(item)
}

#[always_context]
#[proc_macro]
#[anyhow_result]
#[no_context]
pub fn table_join_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = macros::table_join(item)?;
    panic!("{}", result);
}

#[always_context]
#[proc_macro_derive(DatabaseSetup)]
#[anyhow_result]
pub fn database_setup(item: TokenStream) -> anyhow::Result<TokenStream> {
    derive::database_setup(item)
}

#[always_context]
#[proc_macro_derive(Output, attributes(sql))]
#[anyhow_result]
pub fn output(item: TokenStream) -> anyhow::Result<TokenStream> {
    derive::output(item)
}

#[always_context]
#[proc_macro_derive(OutputDebug, attributes(sql))]
#[anyhow_result]
#[no_context]
pub fn output_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let output = derive::output(item)?;

    panic!("{}", output);
}

#[always_context]
#[proc_macro_derive(Insert, attributes(sql))]
#[anyhow_result]
pub fn insert(item: TokenStream) -> anyhow::Result<TokenStream> {
    derive::insert(item)
}

#[always_context]
#[proc_macro_derive(InsertDebug, attributes(sql))]
#[anyhow_result]
#[no_context]
pub fn insert_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let output = derive::insert(item)?;

    panic!("{}", output);
}

#[always_context]
#[proc_macro_derive(Update, attributes(sql))]
#[anyhow_result]
pub fn sql_update(item: TokenStream) -> anyhow::Result<TokenStream> {
    derive::update(item)
}

#[always_context]
#[proc_macro_derive(UpdateDebug, attributes(sql))]
#[anyhow_result]
#[no_context]
pub fn sql_update_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let output = derive::update(item)?;

    panic!("{}", output);
}

#[always_context]
#[proc_macro_derive(Table, attributes(sql))]
#[anyhow_result]
pub fn table(item: TokenStream) -> anyhow::Result<TokenStream> {
    derive::table(item)
}

#[always_context]
#[proc_macro_derive(TableDebug, attributes(sql))]
#[anyhow_result]
#[no_context]
pub fn table_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let output = derive::table(item)?;

    panic!("{}", output);
}

#[always_context]
#[proc_macro_attribute]
#[anyhow_result]
pub fn sql_convenience(attr: TokenStream, item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_convenience_attr::sql_convenience(attr, item)
}
