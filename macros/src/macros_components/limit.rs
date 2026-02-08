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
            Ok(Limit::Expr(Box::new(expr)))
        } else {
            Err(lookahead.error())
        }
    }
}

#[always_context]
impl Limit {
    pub fn into_query_string(self, data: &mut CollectedData) -> String {
        match self {
            Limit::Literal(s) => {
                data.format_params.push(quote! {#s});

                "{}".to_string()
            }
            Limit::Expr(expr) => {
                // Check if the expression can be converted to i64
                data.checks.push(quote_spanned! {expr.span()=>
                    let _test:i64 = #expr as i64;
                });

                // Add binding for the parameter
                let debug_str = format!("Failed to bind `{}` to LIMIT parameter", quote! {#expr});
                data.binds.push(quote_spanned! {expr.span()=>
                    _easy_sql_args.add(&#expr).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                });

                // Add parameter placeholder
                data.format_params.push(data.driver.parameter_placeholder(
                    data.sql_crate,
                    expr.span(),
                    &data.before_param_n,
                    *data.current_param_n,
                ));
                *data.current_param_n += 1;

                "{}".to_string()
            }
        }
    }
}
