use easy_macros::{
    anyhow::{self, Context},
    helpers::{context, parse_macro_input},
    macros::{always_context, get_attributes},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, punctuated::Punctuated},
};

use super::ty_to_variant;

#[always_context]
pub fn sql_update_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
) -> anyhow::Result<TokenStream> {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names.clone().map(|field| field.to_string());

    let mut update_values = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        update_values.push(ty_to_variant(
            field_name.to_token_stream(),
            &field.ty,
            &quote! {easy_lib::sql},
            true,
        )?);
    }

    Ok(quote! {
        impl easy_lib::sql::SqlUpdate<#table> for #item_name {
            fn updates(&self) -> Vec<(String, easy_lib::sql::SqlValueMaybeRef<'_>)> {
                easy_lib::sql::never::never_fn(|| {
                    //Check for validity
                    let update_instance = easy_lib::sql::never::never_any::<Self>();
                    let mut table_instance = easy_lib::sql::never::never_any::<#table>();

                    #(table_instance.#field_names = update_instance.#field_names;)*
                });
                vec![
                    #((
                        #field_names_str.to_string(),
                        #update_values,
                    ),)*
                ]
            }
        }

    })
}

#[always_context]
pub fn sql_update(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named.named,
        syn::Fields::Unnamed(_) => {
            anyhow::bail!("Unnamed struct fields is not supported")
        }
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };

    let mut table = None;

    for attr in get_attributes!(item, #[sql(table = __unknown__)]) {
        if table.is_some() {
            anyhow::bail!("Only one table attribute is allowed");
        }
        table = Some(attr);
    }

    #[no_context]
    let table = table.with_context(context!("Table attribute is required"))?;

    sql_update_base(&item_name, &fields, &table).map(|e| e.into())
}
