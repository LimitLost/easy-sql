use easy_macros::{
    macros::always_context,
    quote::quote,
    syn::{self, parse::Parse},
};

use crate::{
    sql_crate,
    sql_macros_components::{
        sql_column::SqlColumn, sql_expr::SqlExpr, sql_keyword, sql_limit::SqlLimit,
        sql_order_by::OrderBy,
    },
};

use super::WrappedInput;

struct Input {
    distinct: bool,
    where_: Option<SqlExpr>,
    order_by: Option<Vec<OrderBy>>,
    group_by: Option<Vec<SqlColumn>>,
    having: Option<SqlExpr>,
    limit: Option<SqlLimit>,
}

#[always_context]
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

#[always_context]
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
    let input = syn::parse_macro_input!(item as WrappedInput<Input>);
    let input_table = input.table;
    let input = input.input;

    let mut checks = Vec::new();

    let sql_crate = sql_crate();

    let result = if input.only_where() {
        let where_ = input
            .where_
            .unwrap()
            .into_tokens_with_checks(&mut checks, &sql_crate, true);

        quote! {
            Some((|___t___|{
                #(#checks)*
            },
            #sql_crate::WhereClause{
                conditions: #where_
            }))
        }
    } else {
        let where_ = input
            .where_
            .map(|w| {
                let tokens = w.into_tokens_with_checks(&mut checks, &sql_crate, true);

                quote! {Some(#sql_crate::WhereClause{
                    conditions:#tokens
                })}
            })
            .unwrap_or_else(|| quote! {None});

        let group_by = input
            .group_by
            .map(|g| {
                let tokens = g
                    .into_iter()
                    .map(|el| el.into_tokens_with_checks(&mut checks, &sql_crate));

                quote! {Some(#sql_crate::GroupByClause{order_by: vec![#(#tokens),*]})}
            })
            .unwrap_or_else(|| quote! {None});

        let having = input
            .having
            .map(|h| {
                let tokens = h.into_tokens_with_checks(&mut checks, &sql_crate, true);

                quote! {Some(#sql_crate::HavingClause{conditions: #tokens})}
            })
            .unwrap_or_else(|| quote! {None});

        let order_by = input
            .order_by
            .map(|o| {
                let tokens = o
                    .into_iter()
                    .map(|el| el.into_tokens_with_checks(&mut checks, &sql_crate));

                quote! {Some(#sql_crate::OrderByClause{order_by: vec![#(#tokens),*]})}
            })
            .unwrap_or_else(|| quote! {None});
        let limit = input
            .limit
            .map(|l| {
                let tokens = l.into_tokens_with_checks(&mut checks, &sql_crate);

                quote! {Some(#tokens)}
            })
            .unwrap_or_else(|| quote! {None});

        let distinct = input.distinct;

        let checks_tokens = if let Some(table_ty) = input_table {
            quote! {
                |___t___:#table_ty|{
                    #(#checks)*
                },
            }
        } else {
            quote! {}
        };

        quote! {
            Some((#checks_tokens
            #sql_crate::SelectClauses {
                distinct: #distinct,

                where_: #where_,
                group_by: #group_by,
                having: #having,
                order_by: #order_by,
                limit: #limit,
            }))
        }
    };

    // panic!("{}", result);

    result.into()
}
