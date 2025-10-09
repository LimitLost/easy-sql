use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, punctuated::Punctuated},
};
use easy_macros::{
    helpers::{context, parse_macro_input},
    macros::{always_context, get_attributes, has_attributes},
};

use crate::{easy_lib_crate, sql_crate};

use super::ty_to_variant;

#[always_context]
pub fn sql_update_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
    sql_table_macro: bool,
) -> anyhow::Result<TokenStream> {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names.clone().map(|field| field.to_string());
    let field_names2 = field_names.clone();
    let field_names_str2 = field_names_str.clone();

    let sql_crate = sql_crate();
    let easy_lib_crate = easy_lib_crate();

    let mut update_values = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();

        let bytes_allowed = if sql_table_macro {
            has_attributes!(field, #[sql(bytes)])
        } else {
            true
        };

        let ty_variant = ty_to_variant(
            field_name.to_token_stream(),
            &field.ty,
            &sql_crate,
            bytes_allowed,
        )?;

        update_values.push(quote! {
            #sql_crate::SqlExpr::Value(#ty_variant)
        });
    }

    Ok(quote! {
        impl #sql_crate::SqlUpdate<#table> for #item_name {
            fn updates(&mut self) -> #easy_lib_crate::anyhow::Result<Vec<(String, #sql_crate::SqlExpr<'_>)>> {
                #sql_crate::never::never_fn(|| {
                    //Check for validity
                    let update_instance = #sql_crate::never::never_any::<Self>();
                    let mut table_instance = #sql_crate::never::never_any::<#table>();

                    #(table_instance.#field_names = update_instance.#field_names;)*
                });
                Ok(vec![
                    #((
                        #field_names_str.to_string(),
                        #update_values,
                    ),)*
                ])
            }
        }

        impl #sql_crate::SqlUpdate<#table> for &#item_name {
            fn updates(&mut self) -> #easy_lib_crate::anyhow::Result<Vec<(String, #sql_crate::SqlExpr<'_>)>> {
                #sql_crate::never::never_fn(|| {
                    //Check for validity
                    let update_instance = #sql_crate::never::never_any::<#item_name>();
                    let mut table_instance = #sql_crate::never::never_any::<#table>();

                    #(table_instance.#field_names2 = update_instance.#field_names2;)*
                });
                Ok(vec![
                    #((
                        #field_names_str2.to_string(),
                        #update_values,
                    ),)*
                ])
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

    sql_update_base(&item_name, &fields, &table, false).map(|e| e.into())
}
