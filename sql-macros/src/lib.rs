mod sql_macros;
mod sql_macros_components;

mod sql_derive;

use easy_macros::{anyhow, macros::macro_result};
use proc_macro::TokenStream;

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    sql_macros::sql(item)
}
#[proc_macro]
pub fn sql_where(item: TokenStream) -> TokenStream {
    sql_macros::sql_where(item)
}

#[proc_macro_derive(DatabaseSetup)]
#[macro_result]
pub fn database_setup(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::database_setup(item)
}

#[proc_macro_derive(SqlOutput, attributes(sql))]
#[macro_result]
pub fn sql_output(item: TokenStream) -> anyhow::Result<TokenStream> {
    sql_derive::sql_output(item)
}
