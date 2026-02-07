use super::{
    CollectedData,
    expr::Expr,
    keyword::{self},
};
use ::syn::{self, parse::Parse};
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
    pub fn into_query_string(self, data: &mut CollectedData) -> String {
        let expr_query = self.expr.into_query_string(data, false, false);
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
