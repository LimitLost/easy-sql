mod sql;
mod sql_where;

use easy_macros::{
    macros::always_context,
    syn::{self, parse::Parse},
};
pub use sql::*;
pub use sql_where::*;

pub struct WrappedInput<T: Parse> {
    table: syn::Type,
    input: T,
}

#[always_context]
impl<T: Parse> Parse for WrappedInput<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let table = input.parse::<syn::Type>()?;
        let input = input.parse::<T>()?;
        Ok(WrappedInput { table, input })
    }
}
