use crate::macros_components::{
    column::Column, expr::Expr, keyword, limit::Limit, order_by::OrderBy, set::SetExpr,
};
use easy_macros::always_context;
use syn::{self, parse::Parse};

/// Represents the different query types supported by query! macro
#[derive(Debug, Clone)]
pub enum QueryType {
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
    Exists(ExistsQuery),
}

/// SELECT OutputType FROM TableType [WHERE ...] [ORDER BY ...] [LIMIT ...]
#[derive(Debug, Clone)]
pub struct SelectQuery {
    pub output: ReturningData,
    pub table_type: syn::Type,
    pub where_clause: Option<Expr>,
    pub order_by: Option<Vec<OrderBy>>,
    pub group_by: Option<Vec<Column>>,
    pub having: Option<Expr>,
    pub limit: Option<Limit>,
    pub distinct: bool,
}

/// INSERT INTO TableType VALUES {data} [RETURNING OutputType]
#[derive(Debug, Clone)]
pub struct InsertQuery {
    pub table_type: syn::Type,
    pub values: syn::Expr,
    pub returning: Option<ReturningData>,
}

/// UPDATE TableType SET field = value [WHERE ...] [RETURNING OutputType]
#[derive(Debug, Clone)]
pub struct UpdateQuery {
    pub table_type: syn::Type,
    pub set_clause: SetClause,
    pub where_clause: Option<Expr>,
    pub returning: Option<ReturningData>,
}
#[derive(Debug, Clone)]
pub enum SetClause {
    FromType(syn::Expr),
    Expr(SetExpr),
}

/// DELETE FROM TableType [WHERE ...] [RETURNING OutputType]
#[derive(Debug, Clone)]
pub struct DeleteQuery {
    pub table_type: syn::Type,
    pub where_clause: Option<Expr>,
    pub returning: Option<ReturningData>,
}

#[derive(Debug, Clone)]
pub struct ReturningData {
    pub output_type: syn::Type,
    pub output_args: Option<Vec<Expr>>,
}

#[always_context]
impl Parse for ReturningData {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let output_type = input.parse::<syn::Type>()?;
        let output_args = if input.peek(syn::token::Paren) {
            let inside_paren;
            syn::parenthesized!(inside_paren in input);
            let mut args = Vec::new();
            while !inside_paren.is_empty() {
                let arg: Expr = inside_paren.parse()?;
                args.push(arg);

                if inside_paren.peek(syn::Token![,]) {
                    inside_paren.parse::<syn::Token![,]>()?;
                } else {
                    break;
                }
            }
            Some(args)
        } else {
            None
        };

        Ok(ReturningData {
            output_type,
            output_args,
        })
    }
}

/// EXISTS TableType [WHERE ...] [GROUP BY ...] [HAVING ...] [ORDER BY ...] [LIMIT ...]
#[derive(Debug, Clone)]
pub struct ExistsQuery {
    pub table_type: syn::Type,
    pub where_clause: Option<Expr>,
    pub group_by: Option<Vec<Column>>,
    pub having: Option<Expr>,
    pub order_by: Option<Vec<OrderBy>>,
    pub limit: Option<Limit>,
}

#[always_context]
impl Parse for QueryType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(keyword::select) {
            let select_query = input.parse::<SelectQuery>()?;
            Ok(QueryType::Select(select_query))
        } else if lookahead.peek(keyword::insert) {
            let insert_query = input.parse::<InsertQuery>()?;
            Ok(QueryType::Insert(insert_query))
        } else if lookahead.peek(keyword::update) {
            let update_query = input.parse::<UpdateQuery>()?;
            Ok(QueryType::Update(update_query))
        } else if lookahead.peek(keyword::delete) {
            let delete_query = input.parse::<DeleteQuery>()?;
            Ok(QueryType::Delete(delete_query))
        } else if lookahead.peek(keyword::exists) {
            let exists_query = input.parse::<ExistsQuery>()?;
            Ok(QueryType::Exists(exists_query))
        } else {
            Err(lookahead.error())
        }
    }
}

#[always_context]
impl Parse for SelectQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keyword::select>()?;

        // Check for DISTINCT
        let distinct = input.peek(keyword::distinct);
        if distinct {
            input.parse::<keyword::distinct>()?;
        }

        let output = input.parse::<ReturningData>()?;

        input.parse::<keyword::from>()?;
        let table_type = input.parse::<syn::Type>()?;

        let mut where_clause = None;
        let mut order_by = None;
        let mut group_by = None;
        let mut having = None;
        let mut limit = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if where_clause.is_none() && lookahead.peek(keyword::where_) {
                input.parse::<keyword::where_>()?;
                where_clause = Some(input.parse()?);
            } else if order_by.is_none() && lookahead.peek(keyword::order) {
                input.parse::<keyword::order>()?;
                input.parse::<keyword::by>()?;
                let mut order_by_list = Vec::new();
                loop {
                    let order_by_item: OrderBy = input.parse()?;
                    order_by_list.push(order_by_item);
                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    } else {
                        break;
                    }
                }
                order_by = Some(order_by_list);
            } else if group_by.is_none() && lookahead.peek(keyword::group) {
                input.parse::<keyword::group>()?;
                input.parse::<keyword::by>()?;
                let mut group_by_list = Vec::new();
                loop {
                    let group_by_item: Column = input.parse()?;
                    group_by_list.push(group_by_item);
                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    } else {
                        break;
                    }
                }
                group_by = Some(group_by_list);
            } else if having.is_none() && lookahead.peek(keyword::having) {
                input.parse::<keyword::having>()?;
                having = Some(input.parse()?);
            } else if limit.is_none() && lookahead.peek(keyword::limit) {
                input.parse::<keyword::limit>()?;
                limit = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(SelectQuery {
            output,
            table_type,
            where_clause,
            order_by,
            group_by,
            having,
            limit,
            distinct,
        })
    }
}

#[always_context]
impl Parse for InsertQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keyword::insert>()?;
        input.parse::<keyword::into>()?;
        let table_type = input.parse::<syn::Type>()?;
        input.parse::<keyword::values>()?;

        // Parse the expression in braces: {data}
        let inside_braces;
        syn::braced!(inside_braces in input);
        let values = inside_braces.parse::<syn::Expr>()?;

        let mut returning = None;
        if !input.is_empty() {
            let lookahead = input.lookahead1();
            if lookahead.peek(keyword::returning) {
                input.parse::<keyword::returning>()?;
                returning = Some(input.parse::<ReturningData>()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(InsertQuery {
            table_type,
            values,
            returning,
        })
    }
}

#[always_context]
impl Parse for UpdateQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keyword::update>()?;
        let table_type = input.parse::<syn::Type>()?;
        input.parse::<keyword::set>()?;

        // Check if SET expression is in braces or inline
        let set_clause = if input.peek(syn::token::Brace) {
            // Parse the SET expression in braces: {data}
            let inside_braces;
            syn::braced!(inside_braces in input);
            let type_expr = inside_braces.parse::<syn::Expr>()?;
            SetClause::FromType(type_expr)
        } else {
            let set_expr = input.parse::<SetExpr>()?;
            SetClause::Expr(set_expr)
        };

        let mut where_clause = None;
        let mut returning = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if where_clause.is_none() && lookahead.peek(keyword::where_) {
                input.parse::<keyword::where_>()?;
                where_clause = Some(input.parse()?);
            } else if returning.is_none() && lookahead.peek(keyword::returning) {
                input.parse::<keyword::returning>()?;
                returning = Some(input.parse::<ReturningData>()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(UpdateQuery {
            table_type,
            set_clause,
            where_clause,
            returning,
        })
    }
}

#[always_context]
impl Parse for DeleteQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keyword::delete>()?;
        input.parse::<keyword::from>()?;
        let table_type = input.parse::<syn::Type>()?;

        let mut where_clause = None;
        let mut returning = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if where_clause.is_none() && lookahead.peek(keyword::where_) {
                input.parse::<keyword::where_>()?;
                where_clause = Some(input.parse()?);
            } else if returning.is_none() && lookahead.peek(keyword::returning) {
                input.parse::<keyword::returning>()?;
                returning = Some(input.parse::<ReturningData>()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(DeleteQuery {
            table_type,
            where_clause,
            returning,
        })
    }
}

#[always_context]
impl Parse for ExistsQuery {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        input.parse::<keyword::exists>()?;
        let table_type = input.parse::<syn::Type>()?;

        let mut where_clause = None;
        let mut group_by = None;
        let mut having = None;
        let mut order_by = None;
        let mut limit = None;

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if where_clause.is_none() && lookahead.peek(keyword::where_) {
                input.parse::<keyword::where_>()?;
                where_clause = Some(input.parse()?);
            } else if group_by.is_none() && lookahead.peek(keyword::group) {
                input.parse::<keyword::group>()?;
                input.parse::<keyword::by>()?;
                let mut group_by_list = Vec::new();
                loop {
                    let group_by_item: Column = input.parse()?;
                    group_by_list.push(group_by_item);
                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    } else {
                        break;
                    }
                }
                group_by = Some(group_by_list);
            } else if having.is_none() && lookahead.peek(keyword::having) {
                input.parse::<keyword::having>()?;
                having = Some(input.parse()?);
            } else if order_by.is_none() && lookahead.peek(keyword::order) {
                input.parse::<keyword::order>()?;
                input.parse::<keyword::by>()?;
                let mut order_by_list = Vec::new();
                loop {
                    let order_by_item: OrderBy = input.parse()?;
                    order_by_list.push(order_by_item);
                    if input.peek(syn::Token![,]) {
                        input.parse::<syn::Token![,]>()?;
                    } else {
                        break;
                    }
                }
                order_by = Some(order_by_list);
            } else if limit.is_none() && lookahead.peek(keyword::limit) {
                input.parse::<keyword::limit>()?;
                limit = Some(input.parse()?);
            } else {
                return Err(lookahead.error());
            }
        }

        Ok(ExistsQuery {
            table_type,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
        })
    }
}
