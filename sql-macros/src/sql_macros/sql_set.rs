use ::{
    quote::quote,
    syn::{self, parse::Parse},
};

use crate::{sql_crate, sql_macros::WrappedInput, sql_macros_components::sql_expr::SqlExpr};

pub struct SetExpr {
    updates: Vec<(syn::Ident, SqlExpr)>,
}

impl Parse for SetExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut updates = Vec::new();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<syn::Token![=]>()?;
            let where_expr: SqlExpr = input.parse()?;
            if !input.is_empty() {
                input.parse::<syn::Token![,]>()?;
            }
            updates.push((ident, where_expr));
        }

        Ok(SetExpr { updates })
    }
}

pub fn sql_set(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as WrappedInput<SetExpr>);
    let input_table = input.table;

    let input = input.input;
    let mut checks = Vec::new();
    let mut set_updates = Vec::new();

    let sql_crate = sql_crate();

    for (column, where_expr) in input.updates {
        let where_expr_parsed = where_expr.into_tokens_with_checks(&mut checks, &sql_crate, false);
        let column_str = column.to_string();
        checks.push(quote! {
            ___t___.#column;
        });
        set_updates.push(quote! {
            (#column_str.to_string(), #where_expr_parsed)
        });
    }

    let checks_tokens = if let Some(table_ty) = input_table {
        quote! {
            let _ = |___t___:#table_ty|{
                #(#checks)*
            };
        }
    } else {
        quote! {}
    };

    let result = quote! {
        {
            #checks_tokens
            #sql_crate::UpdateSetClause {
                updates: vec![#(#set_updates,)*]
            }
    }
    };

    //panic!("result: {}", result);

    result.into()
}
