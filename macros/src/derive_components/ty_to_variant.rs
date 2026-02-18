use easy_macros::always_context;
use proc_macro2::TokenStream;
use quote::quote;

#[always_context]
pub fn ty_to_variant(
    current_self: TokenStream,
    field_name: TokenStream,
    bytes: bool,
    crate_prefix: &TokenStream,
) -> anyhow::Result<TokenStream> {
    if bytes {
        Ok(quote! {
            #crate_prefix::macro_support::to_binary(&#current_self.#field_name)?
        })
    } else {
        Ok(quote! {
            #current_self.#field_name
        })
    }
}
