mod sql_convenience_attr;
mod sql_macros;
mod sql_macros_components;

mod sql_derive;

use ::{proc_macro2, quote::quote};
use easy_macros::{
    helpers::find_crate_list,
    macros::{always_context, anyhow_result},
};
use proc_macro::TokenStream;

fn sql_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[("easy-lib", quote! {::sql}), ("easy-sql", quote! {})]) {
        found
    } else {
        quote! {self}
    }
}

fn easy_lib_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[("easy-lib", quote! {})]) {
        found
    } else {
        quote! {}
    }
}

fn easy_macros_helpers_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[
        ("easy-lib", quote! {::helpers}),
        ("easy-macros", quote! {::helpers}),
    ]) {
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
pub fn sql(item: TokenStream) -> TokenStream {
    sql_macros::sql(item)
}

/// Debug version of `sql!` that prints the generated code and panics.
/// Useful for inspecting macro expansion during development.
#[proc_macro]
pub fn sql_debug(item: TokenStream) -> TokenStream {
    let result = sql_macros::sql(item);
    panic!("{}", result);
}

#[always_context]
#[proc_macro]
#[anyhow_result]
pub fn table_join(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_macros::table_join(item)
}

#[always_context]
#[proc_macro]
#[anyhow_result]
#[no_context]
pub fn table_join_debug(item: TokenStream) -> anyhow::Result<TokenStream> {
    let result = sql_macros::table_join(item)?;
    panic!("{}", result);
}

#[always_context]
#[proc_macro_derive(DatabaseSetup)]
#[anyhow_result]
pub fn database_setup(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::database_setup(item)
}

#[always_context]
#[proc_macro_derive(SqlOutput, attributes(sql))]
#[anyhow_result]
pub fn sql_output(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_output(item)
}

#[always_context]
#[proc_macro_derive(SqlInsert, attributes(sql))]
#[anyhow_result]
pub fn sql_insert(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_insert(item)
}

#[always_context]
#[proc_macro_derive(SqlUpdate, attributes(sql))]
#[anyhow_result]
pub fn sql_update(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_update(item)
}

#[always_context]
#[proc_macro_derive(SqlTable, attributes(sql))]
#[anyhow_result]
pub fn sql_table(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_table(item)
}

#[always_context]
#[proc_macro_attribute]
#[anyhow_result]
pub fn sql_convenience(attr: TokenStream, item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_convenience_attr::sql_convenience(attr, item)
}
