use crate::query_macro_components::ProvidedDrivers;

use super::{
    expr::Expr,
    keyword::{self},
};
use ::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{self, parse::Parse},
};
use easy_macros::always_context;

#[derive(Debug, Clone)]
pub enum Order {
    Asc,
    Desc,
}

#[always_context]
impl Parse for Order {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::asc) {
            input.parse::<keyword::asc>()?;
            Ok(Order::Asc)
        } else if lookahead.peek(keyword::desc) {
            input.parse::<keyword::desc>()?;
            Ok(Order::Desc)
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderBy {
    pub expr: Expr,
    pub order: Order,
}

#[always_context]
impl OrderBy {
    pub fn into_tokens_with_checks(
        self,
        checks: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
    ) -> TokenStream {
        // For backward compatibility, we need to extract the column from the expression
        // In the old API, OrderBy only supported columns
        let column_parsed = match self.expr {
            Expr::Value(Value::Column(col)) => col.into_tokens_with_checks(checks, sql_crate),
            _ => {
                checks.push(quote! {
                    compile_error!("ORDER BY with complex expressions is only supported in the new query! macro API");
                });
                quote! { compile_error!("Unsupported") }
            }
        };

        let order_parsed = match self.order {
            Order::Asc => quote! {false},
            Order::Desc => quote! {true},
        };

        quote! {
            #sql_crate::OrderBy{
                descending: #order_parsed,
                column: #column_parsed,
            }
        }
    }

    pub fn into_query_string(
        self,
        checks: &mut Vec<TokenStream>,
        binds: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
        driver: &ProvidedDrivers,
        current_param_n: &mut usize,
        current_format_params: &mut Vec<TokenStream>,
        before_param_n: &mut TokenStream,
        before_format: &mut Vec<TokenStream>,
        output_ty: Option<&TokenStream>,
        main_table_type: &TokenStream,
    ) -> String {
        let expr_query = self.expr.into_query_string(
            binds,
            checks,
            sql_crate,
            driver,
            current_param_n,
            current_format_params,
            before_param_n,
            before_format,
            false,
            false,
            output_ty,
            Some(main_table_type),
        );
        let order_query_str = match self.order {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        };

        format!("{} {}", expr_query, order_query_str)
    }
}

#[always_context]
impl Parse for OrderBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Try to parse as an expression (supports both columns and function calls)
        let expr: Expr = input.parse()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::asc) || lookahead.peek(keyword::desc) {
            let order: Order = input.parse()?;
            Ok(OrderBy { expr, order })
        } else {
            Ok(OrderBy {
                expr,
                order: Order::Asc,
            })
        }
    }
}
