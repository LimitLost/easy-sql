use crate::sql_where::WhereExpr;
use easy_macros::{quote::quote, syn};

pub fn sql_where(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as WhereExpr);
    let mut checks = Vec::new();

    let conditions_parsed = input.into_tokens_with_checks(&mut checks);

    quote! {
        (|___t___|{
            #(#checks)*
        },
        easy_lib::easy_sql::WhereClause{
            conditions: #conditions_parsed
        })
    }
    .into()
}
