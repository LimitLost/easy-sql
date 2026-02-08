use anyhow::Context;
use easy_macros::always_context;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

struct DefineSupportsFnTraitInput {
    trait_name: Ident,
    fn_name: LitStr,
}

impl Parse for DefineSupportsFnTraitInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let fn_name: LitStr = input.parse()?;
        Ok(Self {
            trait_name,
            fn_name,
        })
    }
}

#[always_context]
pub fn define_supports_fn_trait(input: TokenStream) -> anyhow::Result<TokenStream> {
    let input: DefineSupportsFnTraitInput = syn::parse(input.clone())?;
    let trait_name = input.trait_name;
    let fn_name = input.fn_name;
    let msg = LitStr::new(
        &format!(
            "Driver `{{Self}}` does not support SQL function {} with {{ARGS}} argument(s).",
            fn_name.value()
        ),
        fn_name.span(),
    );

    Ok(quote! {
        #[easy_macros::always_context]
        #[diagnostic::on_unimplemented(message = #msg)]
        pub trait #trait_name<const ARGS: isize>: crate::Driver {}
    }
    .into())
}
