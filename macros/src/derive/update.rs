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

fn option_inner_type(ty: &syn::Type) -> Option<&syn::Type> {
    let type_path = match ty {
        syn::Type::Path(type_path) => type_path,
        _ => return None,
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let args = match &segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args,
        _ => return None,
    };

    let first = args.args.first()?;
    match first {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    }
}

#[always_context]
pub fn sql_update_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    table: &TokenStream,
    drivers: &[syn::Path],
) -> anyhow::Result<TokenStream> {
    let sql_crate = sql_crate();
    let macro_support = quote! { #sql_crate::macro_support };

    let mut update_statements = Vec::new();
    let mut update_statements_ref = Vec::new();
    let mut validity_checks = Vec::new();
    let mut driver_test_values = Vec::new();
    let mut where_clauses_types = Vec::new();

    for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let query_format = format!("{{delimeter}}{}{{delimeter}} = {{}}, ", field_name_str);
        let field_ty = &field.ty;
        let bytes = has_attributes!(field, #[sql(bytes)]);
        if bytes {
            let bound_ty = quote! { Vec<u8> };
            where_clauses_types.push(quote! {
                for<'__easy_sql_x> #bound_ty: #macro_support::Encode<'__easy_sql_x, #macro_support::InternalDriver<D>>,
                #bound_ty: #macro_support::Type<#macro_support::InternalDriver<D>>,
            });
        } else {
            where_clauses_types.push(quote! {
                for<'__easy_sql_x> #field_ty: #macro_support::Encode<'__easy_sql_x, #macro_support::InternalDriver<D>>,
                #field_ty: #macro_support::Type<#macro_support::InternalDriver<D>>,
            });
        }

        let maybe_update =
            has_attributes!(field, #[sql(maybe_update)]) || has_attributes!(field, #[sql(maybe)]);

        let ty_variant_for_checks = ty_to_variant(
            quote! {_self},
            field_name.to_token_stream(),
            bytes,
            &sql_crate,
        )?;

        let debug_format_str =
            format!("Binding field `{}` to query failed", field_name.to_string());
        let debug_format_str_ref = format!(
            "Binding field `{}` (= {{:?}}) to query failed",
            field_name.to_string()
        );

        if maybe_update {
            let outer = option_inner_type(&field.ty).with_context(|| {
                format!(
                    "#[sql(maybe_update)] / #[sql(maybe)] requires `{}` to be an Option<T> field",
                    field_name
                )
            })?;
            let (base_ty, nested_option) = if let Some(inner) = option_inner_type(outer) {
                (inner.clone(), true)
            } else {
                (outer.clone(), false)
            };

            if nested_option {
                if bytes {
                    let bound_ty = quote! { Vec<u8> };
                    where_clauses_types.push(quote! {
                        for<'__easy_sql_x> #bound_ty: #macro_support::Encode<'__easy_sql_x, #macro_support::InternalDriver<D>>,
                        #bound_ty: #macro_support::Type<#macro_support::InternalDriver<D>>,
                    });
                } else {
                    let option_base_ty = quote! { Option<#base_ty> };
                    where_clauses_types.push(quote! {
                        for<'__easy_sql_x> #option_base_ty: #macro_support::Encode<'__easy_sql_x, #macro_support::InternalDriver<D>>,
                        #option_base_ty: #macro_support::Type<#macro_support::InternalDriver<D>>,
                    });
                }
            }

            validity_checks.push(quote! {
                {
                    #[diagnostic::on_unimplemented(
                        message = "#[sql(maybe_update)] / #[sql(maybe)] requires the table field to be Option<T>, and the update field to be either Option<T>, Option<Option<T>>, or T."
                    )]
                    trait __EasySqlMaybeUpdateCompatible {}
                    impl __EasySqlMaybeUpdateCompatible for (Option<#base_ty>, Option<#base_ty>) {}
                    impl __EasySqlMaybeUpdateCompatible for (Option<#base_ty>, Option<Option<#base_ty>>) {}
                    impl __EasySqlMaybeUpdateCompatible for (Option<#base_ty>, #base_ty) {}

                    fn __easy_sql_check_maybe_update<TableField, UpdateField>(
                        _table_field: &TableField,
                        _update_field: &UpdateField,
                    )
                    where
                        (TableField, UpdateField): __EasySqlMaybeUpdateCompatible,
                    {
                    }

                    __easy_sql_check_maybe_update(&table_instance.#field_name, &update_instance.#field_name);
                }
            });

            let driver_value = if bytes {
                quote! {
                    #macro_support::to_binary(&#macro_support::never_any::<Option<#base_ty>>())?
                }
            } else {
                quote! { #macro_support::never_any::<Option<#base_ty>>() }
            };
            driver_test_values.push(driver_value);

            if nested_option {
                let binding_expr = if bytes {
                    quote! { #macro_support::to_binary(&value)? }
                } else {
                    quote! { value }
                };
                update_statements.push(quote! {
                    if let Some(value) = self.#field_name {
                        current_query.push_str(&format!(
                            #query_format,
                            <D as #sql_crate::Driver>::parameter_placeholder(*parameter_n)
                        ));
                        args_list
                            .add(#binding_expr)
                            .map_err(#macro_support::Error::from_boxed)
                            .context(#debug_format_str)?;
                        *parameter_n += 1;
                    }
                });

                let binding_expr_ref = if bytes {
                    quote! { #macro_support::to_binary(value)? }
                } else {
                    quote! { value }
                };
                update_statements_ref.push(quote! {
                    if let Some(value) = &self.#field_name {
                        current_query.push_str(&format!(
                            #query_format,
                            <D as #sql_crate::Driver>::parameter_placeholder(*parameter_n)
                        ));
                        args_list
                            .add(#binding_expr_ref)
                            .map_err(#macro_support::Error::from_boxed)
                            .context(#debug_format_str)?;
                        *parameter_n += 1;
                    }
                });
            } else {
                let binding_expr = if bytes {
                    quote! { #macro_support::to_binary(&update_value)? }
                } else {
                    quote! { update_value }
                };
                update_statements.push(quote! {
                    if let Some(value) = self.#field_name {
                        current_query.push_str(&format!(
                            #query_format,
                            <D as #sql_crate::Driver>::parameter_placeholder(*parameter_n)
                        ));
                        let update_value = Some(value);
                        args_list
                            .add(#binding_expr)
                            .map_err(#macro_support::Error::from_boxed)
                            .context(#debug_format_str)?;
                        *parameter_n += 1;
                    }
                });

                let binding_expr_ref = if bytes {
                    quote! { #macro_support::to_binary(&self.#field_name)? }
                } else {
                    quote! { &self.#field_name }
                };
                update_statements_ref.push(quote! {
                    if self.#field_name.is_some() {
                        current_query.push_str(&format!(
                            #query_format,
                            <D as #sql_crate::Driver>::parameter_placeholder(*parameter_n)
                        ));
                        args_list
                            .add(#binding_expr_ref)
                            .map_err(#macro_support::Error::from_boxed)
                            .context(#debug_format_str)?;
                        *parameter_n += 1;
                    }
                });
            }
        } else {
            let ty_variant = ty_to_variant(
                quote! {self},
                field_name.to_token_stream(),
                bytes,
                &sql_crate,
            )?;
            let debug_value = if bytes {
                quote! { &self.#field_name }
            } else {
                quote! { #ty_variant }
            };

            driver_test_values.push(ty_variant_for_checks.clone());

            validity_checks.push(quote! {
                {
                    #[diagnostic::on_unimplemented(
                        message = "Update fields must match table field types. You can update Option<T> columns with T or Option<T>."
                    )]
                    trait __EasySqlUpdateCompatible {}
                    impl __EasySqlUpdateCompatible for (#field_ty, #field_ty) {}
                    impl __EasySqlUpdateCompatible for (Option<#field_ty>, #field_ty) {}

                    fn __easy_sql_check_update<TableField, UpdateField>(
                        _table_field: &TableField,
                        _update_field: &UpdateField,
                    )
                    where
                        (TableField, UpdateField): __EasySqlUpdateCompatible,
                    {
                    }

                    __easy_sql_check_update(&table_instance.#field_name, &update_instance.#field_name);
                }
            });

            update_statements.push(quote! {
                current_query.push_str(&format!(
                    #query_format,
                    <D as #sql_crate::Driver>::parameter_placeholder(*parameter_n)
                ));
                args_list
                    .add(#ty_variant)
                    .map_err(#macro_support::Error::from_boxed)
                    .context(#debug_format_str)?;
                *parameter_n += 1;
            });

            let binding_expr_ref = if bytes {
                quote! { #macro_support::to_binary(&self.#field_name)? }
            } else {
                quote! { &self.#field_name }
            };
            update_statements_ref.push(quote! {
                current_query.push_str(&format!(
                    #query_format,
                    <D as #sql_crate::Driver>::parameter_placeholder(*parameter_n)
                ));
                args_list
                    .add(#binding_expr_ref)
                    .map_err(#macro_support::Error::from_boxed)
                    .with_context(|| format!(#debug_format_str_ref, #debug_value))?;
                *parameter_n += 1;
            });
        }
    }

    let driver_tests = drivers.iter().map(|driver| {
        quote! {
            let _=|mut args_list:#macro_support::DriverArguments<'a, #driver>|{
                let _self=#macro_support::never_any::<Self>();
                #(
                    args_list.add(#driver_test_values).map_err(#macro_support::Error::from_boxed)?;
                )*
                #macro_support::Result::<()>::Ok(())
            };
        }
    });

    Ok(quote! {
        impl<'a,D:#sql_crate::Driver> #sql_crate::Update<'a,#table, D> for #item_name
        where #(#where_clauses_types)* {

            fn updates(
                self,
                mut args_list: #macro_support::DriverArguments<'a, D>,
                current_query: &mut String,
                parameter_n: &mut usize,
            ) -> #macro_support::Result<#macro_support::DriverArguments<'a, D>>{
                use #macro_support::{Arguments as _, Context as _};

                let _ = || {
                    //Check for validity
                    let update_instance = #macro_support::never_any::<Self>();
                    let mut table_instance = #macro_support::never_any::<#table>();

                    #(#validity_checks)*
                };

                #(#driver_tests)*

                let delimeter = <D as #sql_crate::Driver>::identifier_delimiter();
                let current_query_start_len = current_query.len();

                #(#update_statements)*
                if current_query.len() >= current_query_start_len + 2 {
                    current_query.pop();
                    current_query.pop();
                }
                Ok(args_list)
            }
        }

        impl<'a,D:#sql_crate::Driver> #sql_crate::Update<'a,#table, D> for &'a #item_name
        where #(#where_clauses_types)* {

            fn updates(
                self,
                mut args_list: #macro_support::DriverArguments<'a, D>,
                current_query: &mut String,
                parameter_n: &mut usize,
            ) -> #macro_support::Result<#macro_support::DriverArguments<'a, D>>{
                use #macro_support::{Arguments, Context as _};
                // Validity check needs to be done only once

                let delimeter = <D as #sql_crate::Driver>::identifier_delimiter();


                #(#update_statements_ref)*

                current_query.pop();
                current_query.pop();

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
    let supported_drivers = super::supported_drivers(&item, &compilation_data, true)?;

    sql_update_base(&item_name, fields, &table, &supported_drivers).map(|tokens| tokens.into())
}
