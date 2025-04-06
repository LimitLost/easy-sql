use easy_macros::{
    anyhow,
    helpers::parse_macro_input,
    macros::always_context,
    quote::{ToTokens, quote},
    syn,
};

#[always_context]
pub fn database_setup(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named.named,
        syn::Fields::Unnamed(fields_unnamed) => fields_unnamed.unnamed,
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };

    let fields_mapped=fields.into_iter().enumerate().map(|(index,field)| {
        let field_name=field.ident.map(|e|e.into_token_stream()).unwrap_or_else(||{
            quote! {
                #index
            }
        });
        let field_type=field.ty;

        let field_type_str=field_type.to_token_stream().to_string();

        let context=format!("Field `{}` with type `{}` of struct `{}` ",field_name, field_type_str, item.ident);

        quote! {
            <#field_type as easy_lib::sql::DatabaseSetup>::setup(conn, used_table_names).await.with_context(easy_lib::helpers::context!(#context));
        }
    });

    let item_name = &item.ident;

    Ok(quote! {
        #[easy_lib::async_trait]
        impl easy_lib::sql::DatabaseSetup for #item_name {
            async fn setup(
                conn: &mut (impl easy_lib::sql::EasyExecutor + Send + Sync),
                used_table_names: &mut Vec<String>,
            ) -> easy_lib::anyhow::Result<()> {
                #(
                    #fields_mapped
                )*
                Ok(())
            }
        }
    }
    .into())
}
