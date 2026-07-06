use super::CollectedData;
use ::{
    quote::{quote, quote_spanned},
    syn::{self, parse::Parse, spanned::Spanned},
};
use easy_macros::always_context;

#[derive(Debug, Clone)]
pub enum Limit {
    Literal(i64),
    Expr(Box<syn::Expr>),
}

#[derive(Debug, Clone)]
pub enum Offset {
    Literal(i64),
    Expr(Box<syn::Expr>),
}

enum ParsedBound {
    Literal(i64),
    Expr(Box<syn::Expr>),
}

fn parse_bound_value(input: syn::parse::ParseStream) -> syn::Result<ParsedBound> {
    let lookahead = input.lookahead1();
    if lookahead.peek(syn::LitInt) {
        let lit: syn::LitInt = input.parse()?;
        let value = lit.base10_parse::<i64>()?;
        Ok(ParsedBound::Literal(value))
    } else if lookahead.peek(syn::token::Brace) {
        let inside_braces;
        syn::braced!(inside_braces in input);
        let expr: syn::Expr = inside_braces.parse()?;
        Ok(ParsedBound::Expr(Box::new(expr)))
    } else {
        Err(lookahead.error())
    }
}

#[always_context]
impl Parse for Limit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match parse_bound_value(input)? {
            ParsedBound::Literal(value) => Ok(Limit::Literal(value)),
            ParsedBound::Expr(expr) => Ok(Limit::Expr(expr)),
        }
    }
}

#[always_context]
impl Parse for Offset {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        match parse_bound_value(input)? {
            ParsedBound::Literal(value) => Ok(Offset::Literal(value)),
            ParsedBound::Expr(expr) => Ok(Offset::Expr(expr)),
        }
    }
}

#[always_context]
impl Limit {
    pub fn into_query_string(self, data: &mut CollectedData) -> String {
        match self {
            Limit::Literal(value) => {
                data.format_params.push(quote! {#value});

                "{}".to_string()
            }
            Limit::Expr(expr) => push_bounded_i64_expr("LIMIT", expr, data),
        }
    }
}

#[always_context]
impl Offset {
    pub fn into_query_string(self, data: &mut CollectedData) -> String {
        match self {
            Offset::Literal(value) => {
                data.format_params.push(quote! {#value});

                "{}".to_string()
            }
            Offset::Expr(expr) => push_bounded_i64_expr("OFFSET", expr, data),
        }
    }
}

fn push_bounded_i64_expr(
    clause_name: &'static str,
    expr: Box<syn::Expr>,
    data: &mut CollectedData,
) -> String {
    // Check if the expression can be converted to i64
    data.checks.push(quote_spanned! {expr.span()=>
        let _test:i64 = {#expr} as i64;
    });

    let debug_format = format!(
        "Failed to bind validated {} parameter value = {{}}",
        clause_name
    );

    data.binds.push(quote_spanned! {expr.span()=>
        _easy_sql_args
            .add({#expr})
            .map_err(anyhow::Error::from_boxed)
            .with_context(|| format!(#debug_format, {#expr}))?;
    });

    data.format_params.push(data.driver.parameter_placeholder(
        data.sql_crate,
        expr.span(),
        &data.before_param_n,
        *data.current_param_n,
    ));
    *data.current_param_n += 1;

    "{}".to_string()
}
