use anyhow::Context;
use easy_macros::always_context;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitInt, Token, Type};

struct ImplSupportsFnAnyInput {
    driver: Type,
    target_trait: Ident,
}

impl Parse for ImplSupportsFnAnyInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let driver: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let target = input.parse()?;
        Ok(Self {
            driver,
            target_trait: target,
        })
    }
}

struct ImplSupportsFnInput {
    driver: Type,
    target: Ident,
    args: Vec<LitInt>,
}

impl Parse for ImplSupportsFnInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let driver: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let target = input.parse()?;
        let mut args = Vec::new();

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
            while !input.is_empty() {
                let arg: LitInt = input.parse()?;
                args.push(arg);
                if input.is_empty() {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }

        if args.is_empty() {
            return Err(input.error("Expected at least one argument count"));
        }

        Ok(Self {
            driver,
            target,
            args,
        })
    }
}

#[always_context]
pub fn impl_supports_fn_any(input: TokenStream) -> anyhow::Result<TokenStream> {
    let input: ImplSupportsFnAnyInput = syn::parse(input.clone())?;
    let sql_crate = crate::sql_crate();
    let driver = input.driver;

    let trait_ident = input.target_trait;

    let expanded = quote! {
        impl<const ARGS: isize> #sql_crate::markers::functions::#trait_ident<ARGS> for #driver {}
    };

    Ok(expanded.into())
}

#[always_context]
pub fn impl_supports_fn(input: TokenStream) -> anyhow::Result<TokenStream> {
    let input: ImplSupportsFnInput = syn::parse(input.clone())?;
    let sql_crate = crate::sql_crate();
    let driver = input.driver;
    let trait_ident = input.target;
    let args = input.args;

    let expanded = quote! {
        #(
            impl #sql_crate::markers::functions::#trait_ident<#args> for #driver {}
        )*
    };

    Ok(expanded.into())
}
