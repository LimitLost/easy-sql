use super::ProvidedDrivers;
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
    pub fn into_query_string(
        self,
        checks: &mut Vec<TokenStream>,
        format_args: &mut Vec<TokenStream>,
        binds: &mut Vec<TokenStream>,
        sql_crate: &TokenStream,
        driver: &ProvidedDrivers,
        param_counter: &mut usize,
        before_param_n: &TokenStream,
    ) -> String {
        match self {
            Limit::Literal(s) => {
                format_args.push(quote! {#s});

                "{}".to_string()
            }
            Limit::Expr(expr) => {
                // Check if the expression can be converted to i64
                checks.push(quote_spanned! {expr.span()=>
                    let _test:i64 = #expr as i64;
                });

                // Add binding for the parameter
                let debug_str = format!(
                    "Failed to bind `{}` to LIMIT parameter",
                    quote! {#expr}.to_string()
                );
                binds.push(quote_spanned! {expr.span()=>
                    _easy_sql_args.add(&#expr).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                });

                // Add parameter placeholder
                format_args.push(driver.parameter_placeholder(
                    sql_crate,
                    expr.span(),
                    before_param_n,
                    &*param_counter,
                ));
                *param_counter += 1;

                "{}".to_string()
            }
        }
    }
}
