use easy_macros::{
    proc_macro2::TokenStream,
    quote::quote,
    syn::{self, parse::Parse},
};

pub enum SqlLimit {
    Literal(i64),
    Expr(syn::Expr),
}

impl Parse for SqlLimit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::LitInt) {
            let lit: syn::LitInt = input.parse()?;
            let value = lit.base10_parse::<i64>()?;
            Ok(SqlLimit::Literal(value))
        } else if lookahead.peek(syn::token::Brace) {
            let inside_braces;
            syn::braced!(inside_braces in input);
            let expr: syn::Expr = inside_braces.parse()?;
            Ok(SqlLimit::Expr(expr))
        } else {
            Err(lookahead.error())
        }
    }
}

impl SqlLimit {
    pub fn into_tokens_with_checks(self, checks: &mut Vec<TokenStream>) -> TokenStream {
        match self {
            SqlLimit::Literal(l) => {
                quote! {easy_lib::easy_sql::LimitClause{
                    limit: #l,
                }}
            }
            SqlLimit::Expr(expr) => {
                quote! {easy_lib::easy_sql::LimitClause{
                    limit: {#expr},
                } }
            }
        }
    }
}
