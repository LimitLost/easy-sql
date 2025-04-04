mod sql_macros;
mod sql_macros_components;

mod sql_derive;

use proc_macro::TokenStream;

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    sql_macros::sql(item)
}
#[proc_macro]
pub fn sql_where(item: TokenStream) -> TokenStream {
    sql_macros::sql_where(item)
}
