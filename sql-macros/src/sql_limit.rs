use easy_macros::syn::{self, parse::Parse};

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
