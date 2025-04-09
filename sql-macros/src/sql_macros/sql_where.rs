use crate::{sql_crate, sql_macros_components::sql_where::WhereExpr};
use easy_macros::{quote::quote, syn};

pub fn sql_where(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as WhereExpr);
    let mut checks = Vec::new();

    let sql_crate = sql_crate();

    let conditions_parsed = input.into_tokens_with_checks(&mut checks, &sql_crate);

    let result = quote! {
        (|___t___|{
            #(#checks)*
        },
        #sql_crate::WhereClause{
            conditions: #conditions_parsed
        })
    };

    //panic!("result: {}", result);

    result.into()
}
