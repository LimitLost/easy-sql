use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, parse::Parse, punctuated::Punctuated},
};
use easy_macros::{always_context, context, get_attributes, has_attributes, parse_macro_input};
use easy_sql_compilation_data::CompilationData;

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
    drivers: &[syn::Path],
    defaults: Vec<syn::Ident>,
) -> anyhow::Result<TokenStream> {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names
        .clone()
        .map(|field| field.to_string())
        .collect::<Vec<_>>();

    let sql_crate = sql_crate();
    let macro_support = quote! { #sql_crate::macro_support };

    let mut insert_values = Vec::new();
    let mut insert_values_ref = Vec::new();
    let mut insert_values_support = Vec::new();
    let mut insert_values_debug = Vec::new();
    let mut insert_values_debug_ref = Vec::new();

    for field in fields.iter() {
        let bytes = has_attributes!(field, #[sql(bytes)]);
        let field_name = field.ident.as_ref().unwrap();
        let mapped = ty_to_variant(
            quote! {self},
            field_name.to_token_stream(),
            bytes,
            &sql_crate,
        )?;
        let mapped_support = ty_to_variant(
            quote! {_self},
            field_name.to_token_stream(),
            bytes,
            &sql_crate,
        )?;
        let debug_value = if bytes {
            quote! { &self.#field_name }
        } else {
            quote! { #mapped }
        };
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
            .with_context(|| format!(#debug_format_str_ref, #debug_value))
        });
        let mapped_ref = if bytes {
            quote! { #macro_support::to_binary(&self.#field_name)? }
        } else {
            quote! { &self.#field_name }
        };
        insert_values.push(mapped);
        insert_values_ref.push(mapped_ref);
        insert_values_support.push(mapped_support);
    }

    let insert_driver_tests=drivers.iter().map(|driver|{
        quote! {
            let _=|mut args_list:#macro_support::DriverArguments<'a, #driver>|{
                let _self=#macro_support::never_any::<Self>();
                #(
                    args_list.add(#insert_values_support).map_err(#macro_support::Error::from_boxed)#insert_values_debug_ref?;
                )*
                #macro_support::Result::<()>::Ok(())
            };
        }
    });

    let where_clauses_types = fields
        .iter()
        .map(|field| {
            let bytes = has_attributes!(field, #[sql(bytes)]);
            if bytes {
                let bound_ty = quote! { Vec<u8> };
                quote! {
                    for<'__easy_sql_x> #bound_ty: #macro_support::Encode<'__easy_sql_x, #macro_support::InternalDriver<D>>,
                    #bound_ty: #macro_support::Type<#macro_support::InternalDriver<D>>,
                }
            } else {
                let field_ty = &field.ty;
                quote! {
                    for<'__easy_sql_x> #field_ty: #macro_support::Encode<'__easy_sql_x, #macro_support::InternalDriver<D>>,
                    #field_ty: #macro_support::Type<#macro_support::InternalDriver<D>>,
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        impl<'a,D:#sql_crate::Driver> #sql_crate::Insert<'a,#table,D> for #item_name
        where #(#where_clauses_types)* {
            fn insert_columns() -> Vec<String> {
                let _ = || {
                    //Check for validity
                    #[diagnostic::on_unimplemented(
                        message = "Insert fields must match table field types. You can insert T into Option<T> columns."
                    )]
                    trait __EasySqlInsertValue<TableField> {
                        fn __easy_sql_insert_value(self) -> TableField;
                    }

                    impl<T> __EasySqlInsertValue<T> for T {
                        fn __easy_sql_insert_value(self) -> T {
                            self
                        }
                    }

                    impl<T> __EasySqlInsertValue<Option<T>> for T {
                        fn __easy_sql_insert_value(self) -> Option<T> {
                            Some(self)
                        }
                    }

                    fn __easy_sql_insert_value<TableField, InsertField>(
                        value: InsertField,
                    ) -> TableField
                    where
                        InsertField: __EasySqlInsertValue<TableField>,
                    {
                        value.__easy_sql_insert_value()
                    }

                    let this_instance = #macro_support::never_any::<Self>();

                    #table {
                        #(
                            #defaults: Default::default(),
                        )*
                        #(
                        #field_names: __easy_sql_insert_value(this_instance.#field_names),
                        )*
                    }
                };
                vec![
                    #(
                        #field_names_str.to_string(),
                    )*
                ]
            }

            fn insert_values(
                self,
                mut args_list: #macro_support::DriverArguments<'a, D>,
            ) -> #macro_support::Result<(#macro_support::DriverArguments<'a, D>, usize)> {
                use #macro_support::Context as _;

                use #macro_support::Arguments;

                #(#insert_driver_tests)*

                    #(
                        args_list.add(#insert_values).map_err(#macro_support::Error::from_boxed)#insert_values_debug?;
                    )*


                Ok((args_list, 1))
            }
        }

        impl<'a,D:#sql_crate::Driver> #sql_crate::Insert<'a,#table,D> for &'a #item_name where #(#where_clauses_types)*{
            fn insert_columns() -> Vec<String> {
                // Validity check needs to be done only once since they are compile time
                vec![
                    #(
                        #field_names_str.to_string(),
                    )*
                ]
            }

            fn insert_values(
                self,
                mut args_list: #macro_support::DriverArguments<'a, D>,
            ) -> #macro_support::Result<(#macro_support::DriverArguments<'a, D>, usize)> {
                use #macro_support::Context as _;

                use #macro_support::Arguments;
                #(
                    args_list.add(#insert_values_ref).map_err(#macro_support::Error::from_boxed)#insert_values_debug_ref?;
                )*
                Ok((args_list, 1))
            }

        }

    })
}

#[always_context]
pub fn insert(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
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

    let supported_drivers = super::supported_drivers(&item, &compilation_data, true)?;

    sql_insert_base(
        &item_name,
        &fields,
        &table,
        &supported_drivers,
        defaults.clone(),
    )
    .map(|s| s.into())
}
