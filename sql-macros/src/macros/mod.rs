mod sql;
pub use sql::*;
mod table_join;
pub use table_join::*;
mod query;
pub use query::*;
mod query_lazy;
pub use query_lazy::*;

use syn::{self, parse::Parse};

use easy_macros::always_context;

mod keywords {

    syn::custom_keyword!(debug_info_mode);
}

pub struct WrappedInput<T: Parse> {
    ///None - means debug mode
    table: Option<syn::Type>,
    driver: Option<syn::Path>,
    input: T,
}

#[always_context]
impl<T: Parse> Parse for WrappedInput<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Check for optional driver specification: <Driver>
        let driver = if input.peek(syn::Token![<]) {
            input.parse::<syn::Token![<]>()?;
            let driver = input.parse::<syn::Path>()?;
            input.parse::<syn::Token![>]>()?;
            Some(driver)
        } else {
            None
        };

        let table = if input.peek(keywords::debug_info_mode) {
            let _ = input.parse::<keywords::debug_info_mode>()?;
            if input.peek(syn::Token![|]) {
                input.parse::<syn::Token![|]>()?;
                input.parse::<syn::Type>()?;
                input.parse::<syn::Token![|]>()?;
            }
            None
        } else {
            if !input.peek(syn::Token![|]) {
                return Err(input.error(format!("Use `easy_sql::build` function (build feature enabled) in build.rs or set current table struct by yourself, syntax: <sql_macro>!(|Table| ...) or <sql_macro>!(<Driver> |Table| ...)")));
            }
            let _ = input.parse::<syn::Token![|]>()?;
            let table = input.parse::<syn::Type>()?;
            let _ = input.parse::<syn::Token![|]>()?;
            Some(table)
        };

        let input = input.parse::<T>()?;
        Ok(WrappedInput {
            table,
            driver,
            input,
        })
    }
}
