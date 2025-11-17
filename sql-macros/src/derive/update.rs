use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, punctuated::Punctuated},
};
use easy_macros::{always_context, context, get_attributes, has_attributes, parse_macro_input};
use sql_compilation_data::CompilationData;

use crate::sql_crate;

use super::ty_to_variant;

#[always_context]
pub fn sql_update_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
    driver: &TokenStream,
) -> anyhow::Result<TokenStream> {
    let field_names = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap())
        .collect::<Vec<_>>();
    let field_names_str = field_names
        .iter()
        .map(|field| field.to_string())
        .collect::<Vec<_>>();

    let sql_crate = sql_crate();
    let macro_support = quote! { #sql_crate::macro_support };

    let mut update_values = Vec::new();
    let mut insert_values_debug = Vec::new();
    let mut insert_values_debug_ref = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();

        let bytes = has_attributes!(field, #[sql(bytes)]);

        let ty_variant = ty_to_variant(field_name.to_token_stream(), bytes, &sql_crate)?;

        let debug_format_str =
            format!("Binding field `{}` to query failed", field_name.to_string());
        let debug_format_str_ref = format!(
            "Binding field `{}` (= {{:?}}) to query failed",
            field_name.to_string()
        );
        insert_values_debug.push(quote! {
            .context(#debug_format_str)
        });
        insert_values_debug_ref.push(quote! {
            .with_context(|| format!(#debug_format_str_ref, #ty_variant))
        });

        update_values.push(ty_variant);
    }

    let sqlx_query_format_str = field_names_str
        .iter()
        .map(|field_name| format!("{{delimeter}}{}{{delimeter}} = {{}}", field_name))
        .collect::<Vec<_>>()
        .join(", ");
    let sqlx_query_format_values = field_names_str
        .iter()
        .enumerate()
        .map(|(i, _)| {
            quote! {
                <#driver as #sql_crate::Driver>::parameter_placeholder(*parameter_n + #i)
            }
        })
        .collect::<Vec<_>>();

    let args_len = field_names.len();

    Ok(quote! {
        impl<'a> #sql_crate::Update<'a,#table, #driver> for #item_name {
            fn updates(self, builder: &mut #sql_crate::QueryBuilder<'_, #driver>) -> #macro_support::Result<Vec<(String, #sql_crate::Expr)>> {
                use #macro_support::Context as _;

                let _ = || {
                    //Check for validity
                    let update_instance = #macro_support::never_any::<Self>();
                    let mut table_instance = #macro_support::never_any::<#table>();

                    #(table_instance.#field_names = update_instance.#field_names;)*
                };
                // Fully safe because we pass by value, not by reference
                unsafe {
                    #(
                        builder.bind(#update_values)#insert_values_debug?;
                    )*
                }

                Ok(vec![
                    #((
                        #field_names_str.to_string(),
                        #sql_crate::Expr::Value,
                    ),)*
                ])
            }

            fn updates_sqlx(
                self,
                mut args_list: #sql_crate::DriverArguments<'a, #driver>,
                current_query: &mut String,
                parameter_n: &mut usize,
            ) -> #macro_support::Result<#sql_crate::DriverArguments<'a, #driver>>{
                use #macro_support::{Arguments, Context as _};

                #(
                    args_list.add(#update_values).map_err(#macro_support::Error::from_boxed)#insert_values_debug?;
                )*

                let delimeter = <#driver as #sql_crate::Driver>::identifier_delimiter();

                current_query.push_str(&format!(
                    #sqlx_query_format_str,
                    #(
                        #sqlx_query_format_values,
                    )*
                ));

                *parameter_n += #args_len;
                Ok(args_list)
            }
        }

        impl<'a> #sql_crate::Update<'a,#table, #driver> for &'a #item_name {
            fn updates( self, builder: &mut #sql_crate::QueryBuilder<'_, #driver>) -> #macro_support::Result<Vec<(String, #sql_crate::Expr)>> {
                use #macro_support::Context as _;
                // Validity check needs to be done only once
                // SAFETY: Fully safe because we pass by reference, and the reference lives until
                // the end of the QueryBuilder usage (parent function call)
                unsafe {
                    #(
                        builder.bind(&#update_values)#insert_values_debug_ref?;
                    )*
                }
                Ok(vec![
                    #((
                        #field_names_str.to_string(),
                        #sql_crate::Expr::Value,
                    ),)*
                ])
            }

            fn updates_sqlx(
                self,
                mut args_list: #sql_crate::DriverArguments<'a, #driver>,
                current_query: &mut String,
                parameter_n: &mut usize,
            ) -> #macro_support::Result<#sql_crate::DriverArguments<'a, #driver>>{
                use #macro_support::{Arguments, Context as _};

                #(
                    args_list.add(&#update_values).map_err(#macro_support::Error::from_boxed)#insert_values_debug_ref?;
                )*

                let delimeter = <#driver as #sql_crate::Driver>::identifier_delimiter();

                current_query.push_str(&format!(
                    #sqlx_query_format_str,
                    #(
                        #sqlx_query_format_values,
                    )*
                ));

                *parameter_n += #args_len;
                Ok(args_list)
            }
        }

    })
}

#[always_context]
pub fn update(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;

    let fields = match &item.fields {
        syn::Fields::Named(fields_named) => &fields_named.named,
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

    let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;
    let supported_drivers = super::supported_drivers(&item, &compilation_data)?;

    let mut result = quote!();
    for driver in supported_drivers.iter() {
        result.extend(sql_update_base(
            &item_name,
            fields,
            &table,
            &driver.to_token_stream(),
        ));
    }

    Ok(result.into())
}
