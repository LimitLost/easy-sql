use easy_macros::{
    quote::quote,
    syn::{self, parse::Parse},
};

use crate::{
    sql_column::SqlColumn, sql_keyword, sql_limit::SqlLimit, sql_order_by::OrderBy,
    sql_where::WhereExpr,
};

enum SqlClause {
    Distinct,
    Where,
    OrderBy,
    GroupBy,
    Having,
    Limit,
}

struct Input {
    distinct: bool,
    where_: Option<WhereExpr>,
    order_by: Option<Vec<OrderBy>>,
    group_by: Option<Vec<SqlColumn>>,
    having: Option<WhereExpr>,
    limit: Option<SqlLimit>,
}

impl Input {
    fn only_where(&self) -> bool {
        let Self {
            distinct,
            where_,
            order_by,
            group_by,
            having,
            limit,
        } = self;
        !distinct
            && where_.is_some()
            && order_by.is_none()
            && group_by.is_none()
            && having.is_none()
            && limit.is_none()
    }
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut distinct = false;
        let mut where_ = None;
        let mut order_by = None;
        let mut group_by = None;
        let mut having = None;
        let mut limit = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if !distinct && lookahead.peek(sql_keyword::distinct) {
                input.parse::<sql_keyword::distinct>()?;
                distinct = true;
                continue;
            }
            if where_.is_none() && lookahead.peek(sql_keyword::where_) {
                input.parse::<sql_keyword::where_>()?;
                where_ = Some(input.parse()?);
                continue;
            }
            if order_by.is_none() && lookahead.peek(sql_keyword::order) {
                input.parse::<sql_keyword::order>()?;
                input.parse::<sql_keyword::by>()?;
                let mut order_by_list = Vec::new();
                while !input.is_empty() {
                    let order_by_item: OrderBy = input.parse()?;
                    order_by_list.push(order_by_item);
                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    } else {
                        break;
                    }
                }
                order_by = Some(order_by_list);
                continue;
            }
            if group_by.is_none() && lookahead.peek(sql_keyword::group) {
                input.parse::<sql_keyword::group>()?;
                input.parse::<sql_keyword::by>()?;
                let mut group_by_list = Vec::new();
                while !input.is_empty() {
                    let group_by_item: SqlColumn = input.parse()?;
                    group_by_list.push(group_by_item);
                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    } else {
                        break;
                    }
                }
                group_by = Some(group_by_list);
                continue;
            }
            if having.is_none() && lookahead.peek(sql_keyword::having) {
                input.parse::<sql_keyword::having>()?;
                having = Some(input.parse()?);
                continue;
            }
            if limit.is_none() && lookahead.peek(sql_keyword::limit) {
                input.parse::<sql_keyword::limit>()?;
                limit = Some(input.parse()?);
                continue;
            }
            return Err(lookahead.error());
        }
        Ok(Input {
            distinct,
            where_,
            order_by,
            group_by,
            having,
            limit,
        })
    }
}

pub fn sql(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as Input);

    if input.only_where() {
        quote! {
            (|t|{

            },
            easy_lib::easy_sql::WhereClause{
                conditions
            })
        }
    } else {
        quote! {
            (|t|{

            },easy_lib::easy_sql::SelectClauses {

            })
        }
    }
}
