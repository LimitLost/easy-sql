mod sql;
mod sql_where;

use easy_macros::{
    macros::always_context,
    syn::{self, parse::Parse},
};
pub use sql::*;
pub use sql_where::*;

mod keywords {
    use easy_macros::syn;

    syn::custom_keyword!(debug_info_mode);
}

pub struct WrappedInput<T: Parse> {
    ///None - means debug mode
    table: Option<syn::Type>,
    input: T,
}

#[always_context]
impl<T: Parse> Parse for WrappedInput<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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
                return Err(input.error(format!("Use `easy_lib::sql::build` function (build feature enabled) in build.rs or set current table struct by yourself, syntax: <sql_macro>!(|Table| ...)")));
            }
            let _ = input.parse::<syn::Token![|]>()?;
            let table = input.parse::<syn::Type>()?;
            let _ = input.parse::<syn::Token![|]>()?;
            Some(table)
        };

        let input = input.parse::<T>()?;
        Ok(WrappedInput { table, input })
    }
}
