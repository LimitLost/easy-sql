use super::{
    sql_column::SqlColumn,
    sql_keyword::{self},
};
use easy_macros::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{self, parse::Parse},
};

pub enum Order {
    Asc,
    Desc,
}

impl Parse for Order {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(sql_keyword::asc) {
            input.parse::<sql_keyword::asc>()?;
            Ok(Order::Asc)
        } else if lookahead.peek(sql_keyword::desc) {
            input.parse::<sql_keyword::desc>()?;
            Ok(Order::Desc)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct OrderBy {
    pub column: SqlColumn,
    pub order: Order,
}

impl OrderBy {
    pub fn into_tokens_with_checks(self, checks: &mut Vec<TokenStream>) -> TokenStream {
        let column_parsed = self.column.into_tokens_with_checks(checks);
        let order_parsed = match self.order {
            Order::Asc => quote! {false},
            Order::Desc => quote! {true},
        };

        quote! {
            easy_lib::easy_sql::OrderBy{
                descending: #order_parsed,
                column: #column_parsed,
            }
        }
    }
}

impl Parse for OrderBy {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let column: SqlColumn = input.parse()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(sql_keyword::asc) || lookahead.peek(sql_keyword::desc) {
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
