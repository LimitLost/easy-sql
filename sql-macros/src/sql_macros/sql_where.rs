use crate::{sql_crate, sql_macros_components::sql_where::WhereExpr};
use easy_macros::{quote::quote, syn};

use super::WrappedInput;

pub fn sql_where(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as WrappedInput<WhereExpr>);
    let table_ty = input.table;
    let input = input.input;
    let mut checks = Vec::new();

    let sql_crate = sql_crate();

    let conditions_parsed = input.into_tokens_with_checks(&mut checks, &sql_crate);

    let result = quote! {
        Some((|___t___:#table_ty|{
            #(#checks)*
        },
        #sql_crate::WhereClause{
            conditions: #conditions_parsed
        }))
    };

    // panic!("result: {}", result);

    result.into()
}
