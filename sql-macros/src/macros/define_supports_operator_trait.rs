use anyhow::Context;
use easy_macros::always_context;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, LitStr, Token};

struct DefineSupportsOperatorTraitInput {
    trait_name: Ident,
    operator: LitStr,
}

impl Parse for DefineSupportsOperatorTraitInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let trait_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let operator: LitStr = input.parse()?;
        Ok(Self {
            trait_name,
            operator,
        })
    }
}

#[always_context]
pub fn define_supports_operator_trait(input: TokenStream) -> anyhow::Result<TokenStream> {
    let input: DefineSupportsOperatorTraitInput = syn::parse(input.clone())?;
    let trait_name = input.trait_name;
    let operator = input.operator;
    let msg = LitStr::new(
        &format!(
            "Driver `{{Self}}` does not support SQL operator {}.",
            operator.value()
        ),
        operator.span(),
    );

    Ok(quote! {
        #[easy_macros::always_context]
        #[diagnostic::on_unimplemented(message = #msg)]
        pub trait #trait_name: crate::Driver {}
    }
    .into())
}
