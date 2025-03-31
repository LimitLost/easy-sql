use crate::{sql_column::SqlColumn, sql_keyword};
use easy_macros::syn::{self, parse::Parse};

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
