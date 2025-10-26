use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, parse::Parse, punctuated::Punctuated},
};
use easy_macros::{
    helpers::{TokensBuilder, context, parse_macro_input},
    macros::{always_context, get_attributes, has_attributes},
};
use sql_compilation_data::CompilationData;

use crate::sql_crate;

use super::ty_to_variant;

struct DefaultAttr {
    fields: Punctuated<syn::Ident, syn::Token![,]>,
}

#[always_context]
impl Parse for DefaultAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fields = Punctuated::<syn::Ident, syn::Token![,]>::parse_terminated(&input)?;
        Ok(DefaultAttr { fields })
    }
}

#[always_context]
pub fn sql_insert_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
    driver: &TokenStream,
    defaults: Vec<syn::Ident>,
) -> anyhow::Result<TokenStream> {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names
        .clone()
        .map(|field| field.to_string())
        .collect::<Vec<_>>();

    let sql_crate = sql_crate();

    let mut insert_values = Vec::new();
    let mut insert_values_debug = Vec::new();
    let mut insert_values_debug_ref = Vec::new();

    for field in fields.iter() {
        let bytes = has_attributes!(field, #[sql(bytes)]);
        let field_name = field.ident.as_ref().unwrap();
        let mapped = ty_to_variant(field_name.to_token_stream(), bytes, &sql_crate)?;
        let debug_format_str =
            format!("Binding field `{}` to query failed", field_name.to_string());
        let debug_format_str_ref = format!(
            "Failed to add `{}` (= {{:?}}) to the sqlx arguments list",
            field_name.to_string()
        );
        insert_values_debug.push(quote! {
            .context(#debug_format_str)
        });
        insert_values_debug_ref.push(quote! {
            .with_context(|| format!(#debug_format_str_ref, #mapped))
        });
        insert_values.push(mapped);
    }

    Ok(quote! {
        impl<'a> #sql_crate::SqlInsert<'a,#table,#driver> for #item_name {
            fn insert_columns() -> Vec<String> {
                #sql_crate::never::never_fn(|| {
                    //Check for validity
                    let this_instance = #sql_crate::never::never_any::<Self>();

                    #table {
                        #(
                            #defaults: Default::default(),
                        )*
                        #(
                        #field_names: this_instance.#field_names,
                        )*
                    }
                });
                vec![
                    #(
                        #field_names_str.to_string(),
                    )*
                ]
            }

            fn insert_values(
                self,
                builder: &mut #sql_crate::QueryBuilder<'_, #driver>,
            ) -> anyhow::Result<usize>{
                use #sql_crate::macro_support::Context as _;

                // Fully safe because we pass by value, not by reference
                unsafe{
                    #(
                        builder.bind(#insert_values)#insert_values_debug?;
                    )*
                }
                Ok(1)
            }

            fn insert_values_sqlx(
                self,
                mut args_list: #sql_crate::DriverArguments<'a, #driver>,
            ) -> anyhow::Result<(#sql_crate::DriverArguments<'a, #driver>, usize)> {
                use #sql_crate::macro_support::Context as _;

                use #sql_crate::macro_support::Arguments;
                    #(
                        args_list.add(#insert_values).map_err(anyhow::Error::from_boxed)#insert_values_debug?;
                    )*


                Ok((args_list, 1))
            }
        }

        impl<'a> #sql_crate::SqlInsert<'a,#table,#driver> for &'a #item_name{
            fn insert_columns() -> Vec<String> {
                // Validity check needs to be done only once
                vec![
                    #(
                        #field_names_str.to_string(),
                    )*
                ]
            }

            fn insert_values(
                self,
                builder: &mut #sql_crate::QueryBuilder<'_, #driver>,
            ) -> anyhow::Result<usize>{
                use #sql_crate::macro_support::Context as _;

                // Fully safe because we pass by reference, and the reference lives until
                // the end of the QueryBuilder usage (parent function call)
                unsafe{
                    #(
                        builder.bind(&#insert_values)#insert_values_debug_ref?;
                    )*
                }
                Ok(1)
            }

            fn insert_values_sqlx(
                self,
                mut args_list: #sql_crate::DriverArguments<'a, #driver>,
            ) -> anyhow::Result<(#sql_crate::DriverArguments<'a, #driver>, usize)> {
                use #sql_crate::macro_support::Context as _;

                use #sql_crate::macro_support::Arguments;
                #(
                    args_list.add(&#insert_values).map_err(anyhow::Error::from_boxed)#insert_values_debug_ref?;
                )*
                Ok((args_list, 1))
            }

        }

    })
}

#[always_context]
pub fn sql_insert(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
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

    let mut defaults = Vec::new();

    for attr in get_attributes!(item, #[sql(default = __unknown__)]) {
        let parsed: DefaultAttr = syn::parse2(
            #[context(tokens)]
            attr.clone(),
        )?;
        defaults.extend(parsed.fields.into_iter());
    }

    let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;

    let supported_drivers = super::supported_drivers(&item, &compilation_data)?;

    let mut result = TokensBuilder::default();

    for driver in supported_drivers {
        result.add(sql_insert_base(
            &item_name,
            &fields,
            &table,
            &driver.to_token_stream(),
            defaults.clone(),
        )?);
    }

    Ok(result.finalize().into())
}
