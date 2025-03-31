mod sql;
mod sql_column;
mod sql_keyword;
mod sql_limit;
mod sql_next_clause;
mod sql_order_by;
mod sql_where;

use proc_macro::TokenStream;

#[proc_macro]
pub fn sql(item: TokenStream) -> TokenStream {
    sql::sql(item)
}
