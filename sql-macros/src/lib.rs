mod sql_convenience_attr;
mod sql_macros;
mod sql_macros_components;

mod sql_derive;

use easy_macros::{
    anyhow,
    helpers::find_crate_list,
    macros::{always_context, anyhow_result},
    proc_macro2,
    quote::quote,
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

fn async_trait_crate() -> proc_macro2::TokenStream {
    if let Some(found) = find_crate_list(&[("easy-lib", quote! {}), ("async-trait", quote! {})]) {
        found
    } else {
        quote! {self}
    }
}

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    sql_macros::sql(item)
}
#[proc_macro]
pub fn sql_where(item: TokenStream) -> TokenStream {
    sql_macros::sql_where(item)
}
#[proc_macro]
pub fn sql_where_debug(item: TokenStream) -> TokenStream {
    let result = sql_macros::sql_where(item);
    panic!("{}", result);
}

#[proc_macro]
pub fn sql_set(item: TokenStream) -> TokenStream {
    sql_macros::sql_set(item)
}
#[proc_macro]
pub fn sql_set_debug(item: TokenStream) -> TokenStream {
    let result = sql_macros::sql_set(item);
    panic!("{}", result);
}

#[proc_macro]
pub fn table_join(item: TokenStream) -> TokenStream {
    sql_macros::table_join(item)
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
