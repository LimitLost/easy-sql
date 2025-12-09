use super::{
    column::Column,
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
    pub column: Column,
    pub order: Order,
}

#[always_context]
impl OrderBy {
    pub fn into_tokens_with_checks(
        self,
        checks: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
    ) -> TokenStream {
        let column_parsed = self.column.into_tokens_with_checks(checks, sql_crate);
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
        sql_crate: &TokenStream,
        format_args: &mut Vec<TokenStream>,
    ) -> String {
        let column_query = self
            .column
            .into_query_string(checks, sql_crate, format_args);
        let order_query_str = match self.order {
            Order::Asc => "ASC",
            Order::Desc => "DESC",
        };

        format!("{} {}", column_query, order_query_str)
    }
}

#[always_context]
impl Parse for OrderBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let column: Column = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::asc) || lookahead.peek(keyword::desc) {
            let order: Order = input.parse()?;
            Ok(OrderBy { column, order })
        } else {
            Ok(OrderBy {
                column,
                order: Order::Asc,
            })
        }
    }
}
