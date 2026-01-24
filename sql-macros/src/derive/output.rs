use std::collections::BTreeSet;

use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, parse::Parse, punctuated::Punctuated},
};

use easy_macros::{
    TokensBuilder, always_context, context, get_attributes, has_attributes, parse_macro_input,
};
use quote::quote_spanned;
use sql_compilation_data::CompilationData;

use crate::{
    CUSTOM_SELECT_ALIAS_PREFIX,
    macros_components::{expr::Expr, joined_field::JoinedField},
    query_macro_components::ProvidedDrivers,
    sql_crate,
};

#[always_context]
pub fn sql_output_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    joined_fields: Vec<JoinedField>,
    table: &TokenStream,
    drivers: &[syn::Path],
) -> anyhow::Result<TokenStream> {
    let sql_crate = sql_crate();
    let macro_support = quote! { #sql_crate::macro_support };

    // === Process custom select attributes FIRST ===
    // Separate fields into regular fields and fields with #[sql(select = ...)]
    struct FieldWithSelect {
        field: syn::Field,
        expr: Expr,
    }

    let mut regular_fields = Punctuated::<syn::Field, syn::Token![,]>::new();
    let mut fields_with_select = Vec::<FieldWithSelect>::new();

    for field in fields.clone() {
        let mut select_attr = None;
        for attr_tokens in get_attributes!(field, #[sql(select = __unknown__)]) {
            if select_attr.is_some() {
                anyhow::bail!(
                    "Only one #[sql(select = ...)] attribute is allowed per field: {}",
                    field.ident.as_ref().unwrap()
                );
            }
            select_attr = Some(attr_tokens);
        }

        if let Some(attr_tokens) = select_attr {
            let parsed_attr: SelectAttribute = syn::parse2(attr_tokens.clone())?;
            fields_with_select.push(FieldWithSelect {
                field: field.clone(),
                expr: parsed_attr.expr,
            });
        } else {
            regular_fields.push(field);
        }
    }

    let mut indices = BTreeSet::new();

    for fws in fields_with_select.iter() {
        fws.expr.collect_indices_impl(&mut indices);
    }

    // Check if any custom select expressions exist (with or without arguments)
    let has_custom_select = !fields_with_select.is_empty();
    let has_custom_select_args = !indices.is_empty();

    let joined_field_aliases = (0..joined_fields.len())
        .into_iter()
        .map(|i| format!("___easy_sql_joined_field_{}", i))
        .collect::<Vec<_>>();

    let joined_checks_field_names = joined_fields
        .iter()
        .map(|joined_field| {
            let field_name = joined_field.field.ident.as_ref().unwrap();
            field_name
        })
        .collect::<Vec<_>>();

    let context_strs2 = joined_fields
        .iter()
        .map(|joined_field| {
            format!(
                "Getting joined field `{}` with type {} for struct `{}` from table `{}`",
                joined_field.field.ident.as_ref().unwrap(),
                joined_field.field.ty.to_token_stream(),
                item_name,
                joined_field.table.to_token_stream()
            )
        })
        .collect::<Vec<_>>();

    let mut fields_quotes: Vec<Box<dyn Fn(&syn::Path) -> TokenStream>> = Vec::new();

    //Handle regular fields (without custom select)
    for field in regular_fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let context_str = format!(
            "Getting field `{}` with type {} for struct `{}`",
            field.ident.as_ref().unwrap(),
            field.ty.to_token_stream(),
            item_name
        );

        let sql_crate = &sql_crate;
        let macro_support = &macro_support;

        if has_attributes!(field, #[sql(bytes)]) {
            let context_str2 = format!(
                "Getting field `{}` with type {} for struct `{}` (Converting from binary)",
                field.ident.as_ref().unwrap(),
                field.ty.to_token_stream(),
                item_name
            );

            fields_quotes.push(Box::new(move |driver|quote! {
                #field_name: #sql_crate::from_binary_vec( <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #field_name_str).with_context(
                    #macro_support::context!(#context_str),
                )?).with_context(
                    #macro_support::context!(#context_str2),
                )?,
            }));
        } else {
            fields_quotes.push(Box::new(move |driver|quote! {
                #field_name: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #field_name_str).with_context(
                    #macro_support::context!(#context_str),
                )?,
            }));
        }
    }

    //Handle fields with custom select
    for field_with_sel in &fields_with_select {
        let field = &field_with_sel.field;
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        // Use the aliased name when reading from the database
        let aliased_name = format!("{}{}", CUSTOM_SELECT_ALIAS_PREFIX, field_name_str);
        let context_str = format!(
            "Getting field `{}` with type {} for struct `{}` (with custom select)",
            field_name,
            field.ty.to_token_stream(),
            item_name
        );
        let sql_crate = &sql_crate;
        let macro_support = &macro_support;

        // Custom select fields are read using their aliased column names
        // The custom SQL expression is used in the SELECT clause with an AS alias,
        // and we read the result from the aliased column
        if has_attributes!(field, #[sql(bytes)]) {
            let context_str2 = format!(
                "Getting field `{}` with type {} for struct `{}` (Converting from binary)",
                field_name,
                field.ty.to_token_stream(),
                item_name
            );

            fields_quotes.push(Box::new(move |driver|quote! {
                #field_name: #sql_crate::from_binary_vec( <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #aliased_name).with_context(
                    #macro_support::context!(#context_str),
                )?).with_context(
                    #macro_support::context!(#context_str2),
                )?,
            }));
        } else {
            fields_quotes.push(Box::new(move |driver|quote! {
                #field_name: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #aliased_name).with_context(
                    #macro_support::context!(#context_str),
                )?,
            }));
        }
    }

    let select_str = regular_fields
        .iter()
        .map(|field| {
            let field_name = field.ident.as_ref().unwrap();
            format!("{{delimeter}}{}{{delimeter}}", field_name)
        })
        .collect::<Vec<_>>()
        .join(", ");

    let select_str_call = if !select_str.is_empty() {
        quote! {
            current_query.push_str(&format!(
                #select_str
            ));
        }
    } else {
        quote! {}
    };

    let select_joined = joined_fields.iter().enumerate().map(|(i, joined_field)| {
        let ref_table = &joined_field.table;
        let table_field = &joined_field.table_field;

        let comma = if i == 0 && select_str.is_empty() {
            ""
        } else {
            ", "
        };

        let alias = format!("___easy_sql_joined_field_{}", i);

        let format_str = format!(
            "{comma}{{delimeter}}{{}}{{delimeter}}.{{delimeter}}{}{{delimeter}} AS {}",
            table_field, alias
        );

        quote! {
            current_query.push_str(&format!(
                #format_str,
                <#ref_table as #sql_crate::Table<D>>::table_name(),
            ));
        }
    });

    // Generate the body for select method
    // Note: select is only called when NormalSelect trait is implemented (no args)
    let select_body = if has_custom_select && !has_custom_select_args {
        // When custom select exists WITHOUT args, delegate to __easy_sql_select
        quote! {
            current_query.push_str(&Self::__easy_sql_select::<D>(delimeter));
        }
    } else {
        // Otherwise, build the select list from regular and joined fields
        // (This covers both: no custom select at all, or custom select with args)
        quote! {
            #select_str_call
            #(#select_joined)*
        }
    };

    // Generate conditional trait implementations based on whether custom select uses args
    let trait_impl = if !has_custom_select_args {
        // No custom select with args - implement NormalSelect
        quote! {
            impl #sql_crate::NormalSelect for #item_name {}
        }
    } else {
        // Custom select with args - implement WithArgsSelect
        quote! {
            impl #sql_crate::WithArgsSelect for #item_name {}
        }
    };

    // Generate __easy_sql_select() method if custom select expressions exist
    let custom_select_impl = if has_custom_select {
        let max_idx = indices.iter().max().copied().unwrap_or_default();

        // Verify no gaps in argument sequence
        if has_custom_select_args {
            for i in 0..=max_idx {
                if !indices.contains(&i) {
                    anyhow::bail!(
                        "Missing argument in #[sql(select = ...)] expressions: arg{} is required but not used. \
                        Used arguments must be sequential starting from arg0.",
                        i
                    );
                }
            }
        }

        // Generate parameter list
        let (arg_params, arg_params_in_call) = if indices.is_empty() {
            (vec![], vec![])
        } else {
            (
                (0..=max_idx)
                    .map(|i| {
                        let arg_name = quote::format_ident!("arg{}", i);
                        quote! { #arg_name: &str }
                    })
                    .collect::<Vec<_>>(),
                (0..=max_idx)
                    .map(|i| {
                        let arg_name = quote::format_ident!("arg{}", i);
                        quote! { #arg_name }
                    })
                    .collect::<Vec<_>>(),
            )
        };

        // Build the SELECT string generation using into_query_string
        let select_generation_code = {
            let mut field_generation = Vec::new();

            // Add regular fields
            for field in regular_fields.iter() {
                let field_name = field.ident.as_ref().unwrap();
                let field_str = field_name.to_string();
                field_generation.push(quote! {
                    parts.push(format!("{delimeter}{}{delimeter}",  #field_str));
                });
            }

            // Add joined fields with their alias
            for (i, _joined_field) in joined_fields.iter().enumerate() {
                let alias: &String = &joined_field_aliases[i];
                let ref_table_ts = &_joined_field.table;
                let field_name = &_joined_field.table_field;

                let parts_format_str = format!(
                    "{{delimeter}}{{}}{{delimeter}}.{{delimeter}}{}{{delimeter}} AS {{delimeter}}{}{{delimeter}}",
                    field_name, alias
                );

                field_generation.push(quote_spanned! {field_name.span()=>
                    let _ = || {
                            let ___t___ = #macro_support::never_any::<#ref_table_ts>();
                            let _ = ___t___.#field_name;
                        };
                    parts.push(format!(#parts_format_str,
                        <#ref_table_ts as #sql_crate::Table<D>>::table_name(),
                    ));
                });
            }

            // Add custom select fields with AS alias
            for field_with_sel in &fields_with_select {
                let field_name = field_with_sel.field.ident.as_ref().unwrap();
                let field_str = field_name.to_string();
                let expr = &field_with_sel.expr;

                // Generate alias with prefix to avoid conflicts
                let alias = format!("{}{}", CUSTOM_SELECT_ALIAS_PREFIX, field_str);

                // Generate the SQL template at compile time
                let mut checks = Vec::new();
                let mut format_params = Vec::new();

                let drivers_for_checks = drivers
                    .iter()
                    .map(|e| e.to_token_stream())
                    .collect::<Vec<_>>();

                // Call into_query_string at proc-macro expansion time with for_custom_select = true
                // Pass the Output type so columns can be validated against it
                let output_type_ts = quote! { #item_name };
                let sql_template = expr.into_query_string(
                    &mut Vec::new(),
                    &mut checks,
                    &sql_crate,
                    &ProvidedDrivers::SingleWithChecks {
                        driver: quote! { D },
                        checks: drivers_for_checks,
                    },
                    &mut 0,
                    &mut format_params,
                    &mut quote! {},
                    &mut Vec::new(),
                    false,
                    true, // for_custom_select
                    Some(&output_type_ts),
                    Some(&table),
                );

                let parts_format_str = format!("{{}} AS {{delimeter}}{}{{delimeter}}", alias);

                // Generate runtime code to format the template with the provided arguments
                // Include compile-time checks for column validity
                field_generation.push(quote! {
                    {
                        // Compile-time validation of columns and types in custom select expression
                        let _ = || {
                            let ___t___ = #macro_support::never_any::<#table>();
                            #(#checks)*
                        };

                        let formatted_expr = format!(#sql_template, #(#format_params),*);
                        parts.push(format!(#parts_format_str, formatted_expr));
                    }
                });
            }

            quote! {
                let mut parts = Vec::new();
                #(#field_generation)*
                parts.join(", ")
            }
        };

        quote! {
            impl #item_name {
                pub fn __easy_sql_select<D: #sql_crate::Driver>(delimeter: &str, #(#arg_params),*) -> String
                where
                    Self: #sql_crate::Output<#table, D>,
                {
                    #select_generation_code
                }

                pub fn __easy_sql_select_driver_from_conn<D: #sql_crate::Driver>(
                    _conn: &impl #sql_crate::EasyExecutor<D>,
                    delimeter: &str,
                    #(#arg_params),*
                ) -> String
                where
                    Self: #sql_crate::Output<#table, D>,
                {
                    Self::__easy_sql_select::<D>(delimeter, #(#arg_params_in_call),*)
                }
            }
        }
    } else {
        quote! {}
    };

    let driver_checks = drivers.iter().map(|driver| {
        let fields_quotes = fields_quotes.iter().map(|f| f(driver));
        quote! {
            let _ = |data: #sql_crate::DriverRow<#driver>| {
                #macro_support::Result::<Self>::Ok(Self {
                    #(
                        #fields_quotes
                    )*
                    #(
                        #joined_checks_field_names: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #joined_field_aliases).with_context(
                            #macro_support::context!(#context_strs2),
                        )?,
                    )*
                })
            };
        }
    }).collect::<Vec<_>>();

    let where_clauses_types=joined_fields.iter().map(|e|&e.field).chain( fields.iter()).map(|field|{
        let field_ty=&field.ty;
        quote! {
            for<'__easy_sql_x> #field_ty: #macro_support::Decode<'__easy_sql_x, #sql_crate::InternalDriver<D>>,
            #field_ty: #macro_support::Type<#sql_crate::InternalDriver<D>>,
        }
    });

    let fields_quotes = fields_quotes.into_iter().map(|f| f(&syn::parse_quote! {D}));

    Ok(quote! {
        impl<D: #sql_crate::Driver> #sql_crate::Output<#table, D> for #item_name
        where #sql_crate::DriverRow<D>: #sql_crate::ToConvert<D>,
        for<'__easy_sql_x> &'__easy_sql_x str: #macro_support::ColumnIndex<#sql_crate::DriverRow<D>>,
        #(#where_clauses_types)*
     {
            type DataToConvert = #sql_crate::DriverRow<D>;
            type UsedForChecks = Self;

            fn select(current_query: &mut String) {
                use #macro_support::Context;
                let delimeter = <D as #sql_crate::Driver>::identifier_delimiter();
                #select_body
            }

            fn convert(data: #sql_crate::DriverRow<D>) -> #macro_support::Result<Self> {
                use #macro_support::{Context,context};

                #(#driver_checks)*

                Ok(Self {
                    #(
                        #fields_quotes
                    )*
                    #(
                        #joined_checks_field_names: <#sql_crate::DriverRow<D> as #sql_crate::SqlxRow>::try_get(&data, #joined_field_aliases).with_context(
                            #macro_support::context!(#context_strs2),
                        )?,
                    )*
                })
            }
        }

        impl #sql_crate::OutputData<#table> for #item_name {
            type SelectProvider = Self;
        }

        #trait_impl
        #custom_select_impl
    })
}

struct FieldAttribute {
    table: syn::Path,
    table_field: syn::Ident,
}

#[always_context]
impl Parse for FieldAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let table = input.parse::<syn::Path>()?;
        input.parse::<syn::Token![.]>()?;
        let table_field = input.parse::<syn::Ident>()?;
        Ok(FieldAttribute { table, table_field })
    }
}

struct SelectAttribute {
    expr: Expr,
}

#[always_context]
impl Parse for SelectAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<Expr>()?;
        Ok(SelectAttribute { expr })
    }
}

#[always_context]
pub fn output(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;

    let fields = match &item.fields {
        syn::Fields::Named(fields_named) => fields_named.named.clone(),
        syn::Fields::Unnamed(_) => {
            anyhow::bail!("Unnamed struct fields is not supported")
        }
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };

    // Get joined fields (fields with #[sql(field = table.field)])
    let mut joined_fields = Vec::new();
    let mut fields2: Punctuated<syn::Field, syn::token::Comma> = Punctuated::new();
    for field in fields.into_iter() {
        //Get attribute #[sql(field = __unknown__)]
        let mut attr = None;
        for a in get_attributes!(field, #[sql(field = __unknown__)]) {
            if attr.is_some() {
                anyhow::bail!("Only one #[sql(field = ...)] attribute is allowed per field!");
            }
            attr = Some(a);
        }
        if let Some(attr) = attr {
            //Parse the attribute
            let attr: FieldAttribute = syn::parse2(attr.clone())?;

            joined_fields.push(JoinedField {
                field,
                table: attr.table,
                table_field: attr.table_field,
            });
        } else {
            fields2.push(field);
        }
    }

    let fields = fields2;

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

    let mut result = TokensBuilder::default();

    result.add(sql_output_base(
        &item_name,
        &fields,
        joined_fields.clone(),
        &table,
        &supported_drivers,
    )?);

    // panic!("{}", result);

    Ok(result.finalize().into())
}
