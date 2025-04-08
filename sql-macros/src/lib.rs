mod sql_macros;
mod sql_macros_components;

mod sql_derive;

use easy_macros::{
    anyhow,
    macros::{always_context, macro_result},
};
use proc_macro::TokenStream;

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    sql_macros::sql(item)
}
#[proc_macro]
pub fn sql_where(item: TokenStream) -> TokenStream {
    sql_macros::sql_where(item)
}

#[always_context]
#[proc_macro_derive(DatabaseSetup)]
#[macro_result]
pub fn database_setup(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::database_setup(item)
}

#[always_context]
#[proc_macro_derive(SqlOutput, attributes(sql))]
#[macro_result]
pub fn sql_output(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_output(item)
}

#[always_context]
#[proc_macro_derive(SqlInsert, attributes(sql))]
#[macro_result]
pub fn sql_insert(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_insert(item)
}

#[always_context]
#[proc_macro_derive(SqlUpdate, attributes(sql))]
#[macro_result]
pub fn sql_update(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_update(item)
}
