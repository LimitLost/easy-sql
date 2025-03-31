use crate::{sql_column::SqlColumn, sql_next_clause::next_clause_token};

use super::sql_keyword;
use easy_macros::{
    quote::{ToTokens, quote},
    syn::{
        self,
        parse::{Lookahead1, Parse},
    },
};

enum AndOr {
    And,
    Or,
}

impl Parse for AndOr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(sql_keyword::and) {
            input.parse::<sql_keyword::and>()?;
            Ok(AndOr::And)
        } else if lookahead.peek(sql_keyword::or) {
            input.parse::<sql_keyword::or>()?;
            Ok(AndOr::Or)
        } else {
            Err(lookahead.error())
        }
    }
}

pub enum WhereExpr {
    Value(SqlValue),
    Parenthesized(Box<WhereExpr>),
    AndOr(Box<WhereExpr>, Vec<(AndOr, WhereExpr)>),
    Not(Box<WhereExpr>),
    IsNull(SqlValue),
    IsNotNull(SqlValue),
    Equal(SqlValue, SqlValue),
    NotEqual(SqlValue, SqlValue),
    GreaterThan(SqlValue, SqlValue),
    GreaterThanOrEqual(SqlValue, SqlValue),
    LessThan(SqlValue, SqlValue),
    LessThanOrEqual(SqlValue, SqlValue),
    Like(SqlValue, SqlValue),
    In(SqlValue, SqlValueIn),
    Between(SqlValue, SqlValue, SqlValue),
}

impl WhereExpr {
    fn into_tokens_with_checks(self) -> easy_macros::proc_macro2::TokenStream {
        match self {
            WhereExpr::Value(sql_value) => todo!(),
            WhereExpr::Parenthesized(where_expr) => todo!(),
            WhereExpr::AndOr(where_expr, items) => todo!(),
            WhereExpr::Not(where_expr) => todo!(),
            WhereExpr::IsNull(sql_value) => todo!(),
            WhereExpr::IsNotNull(sql_value) => todo!(),
            WhereExpr::Equal(sql_value, sql_value1) => todo!(),
            WhereExpr::NotEqual(sql_value, sql_value1) => todo!(),
            WhereExpr::GreaterThan(sql_value, sql_value1) => todo!(),
            WhereExpr::GreaterThanOrEqual(sql_value, sql_value1) => todo!(),
            WhereExpr::LessThan(sql_value, sql_value1) => todo!(),
            WhereExpr::LessThanOrEqual(sql_value, sql_value1) => todo!(),
            WhereExpr::Like(sql_value, sql_value1) => todo!(),
            WhereExpr::In(sql_value, sql_value_in) => todo!(),
            WhereExpr::Between(sql_value, sql_value1, sql_value2) => todo!(),
        }
    }
}

impl ToTokens for WhereExpr {
    fn to_tokens(&self, tokens: &mut easy_macros::proc_macro2::TokenStream) {
        todo!()
    }

    fn into_token_stream(self) -> easy_macros::proc_macro2::TokenStream
    where
        Self: Sized,
    {
        quote! {
            easy_lib::easy_sql::WhereExpr
        }
    }
}

pub enum SqlValue {
    Column(SqlColumn),
    Lit(syn::Lit),
    OutsideVariable(syn::Expr),
}

pub enum SqlValueIn {
    Single(SqlValue),
    Multiple(Vec<SqlValue>),
}

impl SqlValue {
    fn lookahead(l: &Lookahead1<'_>) -> bool {
        l.peek(syn::Ident) || l.peek(syn::Lit) || l.peek(syn::token::Brace)
    }
}

impl Parse for SqlValue {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::Ident) {
            Ok(SqlValue::Column(input.parse()?))
        } else if lookahead.peek(syn::Lit) {
            let lit: syn::Lit = input.parse()?;
            Ok(SqlValue::Lit(lit))
        } else if lookahead.peek(syn::token::Brace) {
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(SqlValue::OutsideVariable(expr))
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for SqlValueIn {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Paren) {
            let inside_paren;
            syn::parenthesized!(inside_paren in input);
            let mut values = Vec::new();
            while !inside_paren.is_empty() {
                let value = inside_paren.parse::<SqlValue>()?;
                values.push(value);
                if inside_paren.is_empty() {
                    break;
                }
                let lookahead2 = inside_paren.lookahead1();
                if lookahead2.peek(syn::Token![,]) {
                    inside_paren.parse::<syn::Token![,]>()?;
                } else {
                    break;
                }
            }
            Ok(SqlValueIn::Multiple(values))
        } else if SqlValue::lookahead(&lookahead) {
            let value = input.parse::<SqlValue>()?;
            Ok(SqlValueIn::Single(value))
        } else {
            Err(lookahead.error())
        }
    }
}

fn continue_parse_expr(
    input: syn::parse::ParseStream,
    current_expr: WhereExpr,
) -> syn::Result<WhereExpr> {
    let lookahead = input.lookahead1();
    if input.is_empty() || next_clause_token(&lookahead) {
        Ok(current_expr)
    } else {
        let first_expr = current_expr;
        let mut next_exprs = vec![];
        while !input.is_empty() {
            let lookahead = input.lookahead1();
            if next_clause_token(&lookahead) {
                break;
            }

            let and_or = input.parse::<AndOr>()?;
            let next = parse_where_expr_no_continue(&input)?;

            next_exprs.push((and_or, next));
        }

        Ok(WhereExpr::AndOr(Box::new(first_expr), next_exprs))
    }
}

fn continue_parse_value_no_expr(
    input: syn::parse::ParseStream,
    current_value: SqlValue,
    lookahead: syn::parse::Lookahead1<'_>,
) -> syn::Result<WhereExpr> {
    if input.is_empty() || next_clause_token(&lookahead) {
        return Ok(WhereExpr::Value(current_value));
    }

    if lookahead.peek(sql_keyword::is) {
        input.parse::<sql_keyword::is>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(sql_keyword::not) {
            input.parse::<sql_keyword::not>()?;
            let lookahead3 = input.lookahead1();
            if lookahead3.peek(sql_keyword::null) {
                input.parse::<sql_keyword::null>()?;
                Ok(WhereExpr::IsNotNull(current_value))
            } else {
                Err(lookahead3.error())
            }
        } else if lookahead2.peek(sql_keyword::null) {
            input.parse::<sql_keyword::null>()?;
            Ok(WhereExpr::IsNull(current_value))
        } else {
            Err(lookahead2.error())
        }
    } else if lookahead.peek(syn::Token![=]) {
        input.parse::<syn::Token![=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::Equal(current_value, right_value))
    } else if lookahead.peek(syn::Token![!=]) {
        input.parse::<syn::Token![!=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::NotEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![>]) {
        input.parse::<syn::Token![>]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::GreaterThan(current_value, right_value))
    } else if lookahead.peek(syn::Token![>=]) {
        input.parse::<syn::Token![>=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::GreaterThanOrEqual(current_value, right_value))
    } else if lookahead.peek(syn::Token![<]) {
        input.parse::<syn::Token![<]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::LessThan(current_value, right_value))
    } else if lookahead.peek(syn::Token![<=]) {
        input.parse::<syn::Token![<=]>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::LessThanOrEqual(current_value, right_value))
    } else if lookahead.peek(sql_keyword::like) {
        input.parse::<sql_keyword::like>()?;
        let right_value = input.parse::<SqlValue>()?;
        Ok(WhereExpr::Like(current_value, right_value))
    } else if lookahead.peek(sql_keyword::in_) {
        input.parse::<sql_keyword::in_>()?;
        let right_value = input.parse::<SqlValueIn>()?;
        Ok(WhereExpr::In(current_value, right_value))
    } else if lookahead.peek(sql_keyword::between) {
        input.parse::<sql_keyword::between>()?;
        let middle_value = input.parse::<SqlValue>()?;
        let lookahead2 = input.lookahead1();
        if lookahead2.peek(sql_keyword::and) {
            input.parse::<sql_keyword::and>()?;
            let right_value = input.parse::<SqlValue>()?;
            Ok(WhereExpr::Between(current_value, middle_value, right_value))
        } else {
            Err(lookahead2.error())
        }
    } else {
        Err(lookahead.error())
    }
}

fn continue_parse_value_maybe_expr(
    input: syn::parse::ParseStream,
    current_value: SqlValue,
) -> syn::Result<WhereExpr> {
    if input.is_empty() {
        return Ok(WhereExpr::Value(current_value));
    }

    let lookahead = input.lookahead1();

    if lookahead.peek(sql_keyword::and) || lookahead.peek(sql_keyword::or) {
        continue_parse_expr(input, WhereExpr::Value(current_value))
    } else {
        continue_parse_value_no_expr(input, current_value, lookahead)
    }
}

fn parse_where_expr_no_continue(input: &syn::parse::ParseStream) -> syn::Result<WhereExpr> {
    let lookahead = input.lookahead1();

    if lookahead.peek(sql_keyword::not) {
        input.parse::<sql_keyword::not>()?;
        let expr = parse_where_expr_no_continue(input)?;
        Ok(WhereExpr::Not(Box::new(expr)))
    } else if lookahead.peek(syn::token::Paren) {
        let inside_paren;
        syn::parenthesized!(inside_paren in input);
        let expr = inside_paren.parse::<WhereExpr>()?;
        Ok(WhereExpr::Parenthesized(Box::new(expr)))
    } else if SqlValue::lookahead(&lookahead) {
        let parsed = input.parse::<SqlValue>()?;

        if input.is_empty() {
            return Ok(WhereExpr::Value(parsed));
        }

        let lookahead2 = input.lookahead1();

        continue_parse_value_no_expr(input, parsed, lookahead2)
    } else {
        Err(lookahead.error())
    }
}

impl Parse for WhereExpr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(sql_keyword::not) {
            input.parse::<sql_keyword::not>()?;
            let expr = input.parse::<WhereExpr>()?;
            return Ok(WhereExpr::Not(Box::new(expr)));
        } else if lookahead.peek(syn::token::Paren) {
            let inside_paren;
            syn::parenthesized!(inside_paren in input);
            let expr = inside_paren.parse::<WhereExpr>()?;
            let where_expr = WhereExpr::Parenthesized(Box::new(expr));
            continue_parse_expr(input, where_expr)
        } else if SqlValue::lookahead(&lookahead) {
            let parsed = input.parse::<SqlValue>()?;

            continue_parse_value_maybe_expr(input, parsed)
        } else {
            Err(lookahead.error())
        }
    }
}
