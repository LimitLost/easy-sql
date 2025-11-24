use ::{
    proc_macro2::TokenStream,
    quote::{quote, quote_spanned},
    syn::{self, parse::Parse, spanned::Spanned},
};
use easy_macros::always_context;

#[derive(Debug, Clone)]
pub enum Limit {
    Literal(i64),
    Expr(syn::Expr),
}

#[always_context]
impl Parse for Limit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitInt) {
            let lit: syn::LitInt = input.parse()?;
            let value = lit.base10_parse::<i64>()?;
            Ok(Limit::Literal(value))
        } else if lookahead.peek(syn::token::Brace) {
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(Limit::Expr(expr))
        } else {
            Err(lookahead.error())
        }
    }
}

#[always_context]
impl Limit {
    pub fn into_tokens_with_checks(
        self,
        _checks: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
    ) -> TokenStream {
        match self {
            Limit::Literal(l) => {
                quote! {#sql_crate::LimitClause{
                    limit: #l as usize,
                }}
            }
            Limit::Expr(expr) => {
                quote_spanned! {expr.span()=>#sql_crate::LimitClause{
                    limit: {#expr},
                } }
            }
        }
    }

    pub fn into_query_string(
        self,
        checks: &mut Vec<TokenStream>,
        format_args: &mut Vec<TokenStream>,
    ) -> String {
        match self {
            Limit::Literal(s) => {
                format_args.push(quote! {#s});

                "{}".to_string()
            }
            Limit::Expr(expr) => {
                format_args.push(quote! {#expr});

                checks.push(quote_spanned! {expr.span()=>
                    let _test:i64 = #expr as i64;
                });

                "{}".to_string()
            }
        }
    }
}
