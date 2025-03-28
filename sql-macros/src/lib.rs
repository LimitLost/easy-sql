mod sql;

use proc_macro::TokenStream;

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    sql::sql(item)
}
