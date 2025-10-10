use crate::{sql_crate, sql_macros_components::sql_expr::SqlExpr};
use ::{quote::quote, syn};

use super::WrappedInput;

pub fn sql_where(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as WrappedInput<SqlExpr>);
    let input_table = input.table;

    let input = input.input;
    let mut checks = Vec::new();

    let sql_crate = sql_crate();

    let driver = quote! {#sql_crate::Sqlite};

    let conditions_parsed = input.into_tokens_with_checks(&mut checks, &sql_crate, true, &driver);

    let checks_tokens = if let Some(table_ty) = input_table {
        quote! {
            |___t___:#table_ty|{
                #(#checks)*
            },
        }
    } else {
        quote! {}
    };

    let result = quote! {
        Some((
            #checks_tokens
        #sql_crate::WhereClause::<#driver>{
            conditions: #conditions_parsed
        }))
    };

    //panic!("result: {}", result);

    result.into()
}
