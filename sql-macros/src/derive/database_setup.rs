use ::{
    anyhow::{self, Context},
    quote::{ToTokens, quote},
    syn,
};

use easy_macros::{TokensBuilder, always_context, parse_macro_input};
use sql_compilation_data::CompilationData;

use crate::sql_crate;

#[always_context]
pub fn database_setup(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);

    let sql_crate = sql_crate();
    let macro_support = quote! { #sql_crate::macro_support };

    let fields = match &item.fields {
        syn::Fields::Named(fields_named) => &fields_named.named,
        syn::Fields::Unnamed(fields_unnamed) => &fields_unnamed.unnamed,
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };

    let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;

    let supported_drivers = super::supported_drivers(&item, &compilation_data)?;

    let mut result = TokensBuilder::default();

    for driver in supported_drivers {
        let fields_mapped=fields.iter().enumerate().map(|(index,field)| {
            let field_name=field.ident.as_ref().map(|e|e.into_token_stream()).unwrap_or_else(||{
                quote! {
                    #index
                }
            });
            let field_type=&field.ty;

            let field_type_str=field_type.to_token_stream().to_string();

            let context=format!("Field `{}` with type `{}` of struct `{}` ",field_name, field_type_str, item.ident);

            quote! {
                <#field_type as #sql_crate::DatabaseSetup<#driver>>::setup(conn).await.with_context(#macro_support::context!(#context))?;
            }
        });

        let item_name = &item.ident;

        result.add(quote! {
            impl #sql_crate::DatabaseSetup<#driver> for #item_name {
                async fn setup(
                    conn: &mut (impl #sql_crate::EasyExecutor<#driver> + Send + Sync)
                ) -> #macro_support::Result<()> {
                    use #macro_support::Context;

                    #(
                        #fields_mapped
                    )*
                    Ok(())
                }
            }
        })
    }

    Ok(result.finalize().into())
}
