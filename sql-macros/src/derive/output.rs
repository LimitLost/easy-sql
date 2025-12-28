use std::collections::BTreeSet;

use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    syn::{self, parse::Parse, punctuated::Punctuated},
};

use easy_macros::{
    context, parse_macro_input, TokensBuilder,
    always_context, get_attributes, has_attributes,
};
use sql_compilation_data::CompilationData;

use crate::{ sql_crate,  macros_components::joined_field::JoinedField, macros_components::expr::Expr, CUSTOM_SELECT_ALIAS_PREFIX
};

#[always_context]
pub fn sql_output_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    joined_fields: Vec<JoinedField>,
    table: &TokenStream,
    driver: &TokenStream,
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

    let mut indices=BTreeSet::new();

    for fws in fields_with_select.iter(){
        fws.expr.collect_indices_impl(&mut indices);
    }
    
    // Check if any custom select expressions exist (with or without arguments)
    let has_custom_select = !fields_with_select.is_empty();
    let has_custom_select_args = !indices.is_empty();

    // Now create iterators from regular_fields (not all fields)
    let field_names = regular_fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names.clone().map(|field| field.to_string());
    
    // Also create iterators for custom select fields
    let custom_select_field_names = fields_with_select.iter().map(|fws| fws.field.ident.as_ref().unwrap());
    let custom_select_field_names_str = custom_select_field_names.clone().map(|field| field.to_string());
    // Create aliased names for custom select fields (used in SQL AS clause and when reading results)
    let custom_select_field_aliases = custom_select_field_names_str.clone().map(|field| {
        format!("{}{}", CUSTOM_SELECT_ALIAS_PREFIX, field)
    }).collect::<Vec<_>>();
    let custom_select_field_names_validity = fields_with_select.iter().map(|fws| fws.field.ident.as_ref().unwrap());

    let joined_field_aliases=(0..joined_fields.len()).into_iter().map(|i|{
        format!("___easy_sql_joined_field_{}",i)
    }).collect::<Vec<_>>();

    let joined_checks = joined_fields.iter().map(|joined_field| {
        let field_name = joined_field.field.ident.as_ref().unwrap();
        let field_type = &joined_field.field.ty;
        let ref_table = &joined_field.table;
        let table_field = &joined_field.table_field;

        //Check if the field type is an option
        let is_option = match field_type {
            syn::Type::Path(path_ty) => {
                if let Some(last_segment) = path_ty.path.segments.last() {
                    //Check if the type is an option
                    if last_segment.ident == "Option" {
                        //Get the inner type
                        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                            args.args.first().is_some()
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            ty => {
                let compile_error =
                    format!("Non path type is not supported `{}`", ty.to_token_stream());
                return quote! {
                    let #field_name={
                        compile_error!(#compile_error);
                        panic!()
                    };
                };
            }
        };

        //Create _Flatten trait if needed
        let (flatten, flatten_use) = if is_option {
            (
                quote! {
                    trait _Flatten{
                        fn _flatten(self)->#field_type;
                    }

                    impl _Flatten for #field_type{
                        fn _flatten(self)->#field_type{
                            self
                        }
                    }
                    impl _Flatten for Option<#field_type>{
                        fn _flatten(self)->#field_type{
                            self.unwrap()
                        }
                    }
                },
                quote! {
                    ._flatten()
                },
            )
        } else {
            (quote! {}, quote! {})
        };

        quote! {
            let #field_name = {
                //Get field from reference table
                let mut table_instance = #macro_support::never_any::<#ref_table>();

                #flatten
                    
                <#table as #sql_crate::HasTableJoined<#ref_table>>::into_maybe_option(table_instance.#table_field)#flatten_use
            };
        }
    }).collect::<Vec<_>>();

    let joined_checks_field_names=joined_fields.iter().map(|joined_field|{
        let field_name = joined_field.field.ident.as_ref().unwrap();
        field_name
    }).collect::<Vec<_>>();

    let joined_checks_table_names=joined_fields.iter().map(|joined_field|{
        joined_field.table.clone()
    }).collect::<Vec<_>>();

    let joined_checks_column_name_format=joined_fields.iter().map(|joined_field|{
        format!("{}",joined_field.table_field) 
    });

    let context_strs2 = joined_fields.iter().map(|joined_field| {
        format!(
            "Getting joined field `{}` with type {} for struct `{}` from table `{}`",
            joined_field.field.ident.as_ref().unwrap(),
            joined_field.field.ty.to_token_stream(),
            item_name,
            joined_field.table.to_token_stream()
        )
    });

    let mut fields_quotes=Vec::new();

    //Handle regular fields (without custom select)
    for field in regular_fields.iter()
    {
        let field_name=field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let context_str=format!(
            "Getting field `{}` with type {} for struct `{}`",
            field.ident.as_ref().unwrap(),
            field.ty.to_token_stream(),
            item_name
        );

        if has_attributes!(field, #[sql(bytes)]){
            let context_str2=format!(
                "Getting field `{}` with type {} for struct `{}` (Converting from binary)",
                field.ident.as_ref().unwrap(),
                field.ty.to_token_stream(),
                item_name
            );

            fields_quotes.push(quote! {
                #field_name: #sql_crate::from_binary_vec( <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #field_name_str).with_context(
                    #macro_support::context!(#context_str),
                )?).with_context(
                    #macro_support::context!(#context_str2),
                )?,
            });
        }else{
            fields_quotes.push(quote! {
                #field_name: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #field_name_str).with_context(
                    #macro_support::context!(#context_str),
                )?,
            });
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

        // Custom select fields are read using their aliased column names
        // The custom SQL expression is used in the SELECT clause with an AS alias,
        // and we read the result from the aliased column
        if has_attributes!(field, #[sql(bytes)]){
            let context_str2=format!(
                "Getting field `{}` with type {} for struct `{}` (Converting from binary)",
                field_name,
                field.ty.to_token_stream(),
                item_name
            );

            fields_quotes.push(quote! {
                #field_name: #sql_crate::from_binary_vec( <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #aliased_name).with_context(
                    #macro_support::context!(#context_str),
                )?).with_context(
                    #macro_support::context!(#context_str2),
                )?,
            });
        }else{
            fields_quotes.push(quote! {
                #field_name: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #aliased_name).with_context(
                    #macro_support::context!(#context_str),
                )?,
            });
        }
    }


    let select_sqlx_str=regular_fields.iter().map(|field|{
        let field_name=field.ident.as_ref().unwrap();
        format!("{{delimeter}}{}{{delimeter}}",field_name)
    }).collect::<Vec<_>>().join(", ");

    let select_sqlx_str_call = if !select_sqlx_str.is_empty() {
        quote! {
            current_query.push_str(&format!(
                #select_sqlx_str
            ));
        }
    } else {
        quote! {}
    };

    let select_sqlx_joined = joined_fields.iter().enumerate().map(|(i,joined_field)| {
        let ref_table = &joined_field.table;
        let table_field = &joined_field.table_field;

        let comma=if i==0 && select_sqlx_str.is_empty() {
            ""
        }else{
            ", "
        };

        let alias=format!("___easy_sql_joined_field_{}",i);

        let format_str=format!(
            "{comma}{{delimeter}}{{}}{{delimeter}}.{{delimeter}}{}{{delimeter}} AS {}",
            table_field,
            alias
        );

        quote! {
            current_query.push_str(&format!(
                #format_str,
                <#ref_table as #sql_crate::Table<#driver>>::table_name(),
            ));
        }
    });
    
    // Generate the body for select_sqlx method
    // Note: select_sqlx is only called when NormalSelect trait is implemented (no args)
    let select_sqlx_body = if has_custom_select && !has_custom_select_args {
        // When custom select exists WITHOUT args, delegate to __easy_sql_select
        quote! {
            current_query.push_str(&Self::__easy_sql_select(delimeter));
        }
    } else {
        // Otherwise, build the select list from regular and joined fields
        // (This covers both: no custom select at all, or custom select with args)
        quote! {
            #select_sqlx_str_call
            #(#select_sqlx_joined)*
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
        let arg_params = if indices.is_empty() {
            vec![]
        } else {
            (0..=max_idx).map(|i| {
                let arg_name = quote::format_ident!("arg{}", i);
                quote! { #arg_name: &str }
            }).collect::<Vec<_>>()
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
                
                // Call into_query_string at proc-macro expansion time with for_custom_select = true
                // Pass the Output type so columns can be validated against it
                let output_type_ts = quote! { #item_name };
                let sql_template = expr.into_query_string(
                    &mut Vec::new(),
                    &mut checks,
                    &sql_crate,
                    driver,
                    &mut 0,
                    &mut format_params,
                    &mut quote! {},
                    &mut Vec::new(),
                    false,
                    true, // for_custom_select
                    Some(&output_type_ts),
                    Some(&table)
                );
                
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
                        parts.push(format!("{} AS {delimeter}{}{delimeter}", formatted_expr, #alias));
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
                pub fn __easy_sql_select(delimeter: &str, #(#arg_params),*) -> String {
                    #select_generation_code
                }
            }
        }
    } else {
        quote! {}
    };

    
        

    Ok(quote! {
        impl #sql_crate::Output<#table, #driver> for #item_name {
            type DataToConvert = #sql_crate::DriverRow<#driver>;
            type UsedForChecks = Self;

            fn sql_to_query<'a>(sql: #sql_crate::Sql, builder: #sql_crate::QueryBuilder<'a, #driver>) -> #macro_support::Result<#sql_crate::QueryData<'a, #driver>> {
                use #macro_support::Context;
                
                let _ = || {
                    //Check for validity
                    let table_instance = #macro_support::never_any::<#table>();

                    //Joined fields check for validity
                    #(#joined_checks)*

                    Self {
                        #(#field_names: table_instance.#field_names,)*
                        //Custom select fields (with dummy values for type checking)
                        #(#custom_select_field_names_validity: #macro_support::never_any(),)*
                        //Joined fields
                        #(#joined_checks_field_names,)*
                    }
                };

                let requested_columns = vec![
                    #(
                        #sql_crate::RequestedColumn {
                            table_name: None,
                            name: #field_names_str.to_owned(),
                            alias: None,
                        },
                    )*
                    #(
                        #sql_crate::RequestedColumn {
                            table_name: None,
                            name: #custom_select_field_names_str.to_owned(),
                            alias: Some(#custom_select_field_aliases.to_owned()),
                        },
                    )*
                    #(
                        #sql_crate::RequestedColumn {
                            table_name: Some(<#joined_checks_table_names as #sql_crate::Table<#driver>>::table_name()),
                            name: #joined_checks_column_name_format.to_owned(),
                            alias: Some(#joined_field_aliases.to_owned()),
                        },
                    )*
                ];

                sql.query_output(builder, requested_columns)
            }
            
            fn select_sqlx(current_query: &mut String) {
                use #macro_support::Context;
                let delimeter = <#driver as #sql_crate::Driver>::identifier_delimiter();
                #select_sqlx_body
            }

            fn convert(data: #sql_crate::DriverRow<#driver>) -> #macro_support::Result<Self> {
                use #macro_support::{Context,context};

                Ok(Self {
                    #(
                        #fields_quotes
                    )*
                    #(
                        #joined_checks_field_names: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #joined_field_aliases).with_context(
                            #macro_support::context!(#context_strs2),
                        )?,
                    )*
                })
            }
        }
        
        #trait_impl
        #custom_select_impl
    })
}

struct FieldAttribute{
    table:syn::Path,
    table_field:syn::Ident,
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
    let mut joined_fields=Vec::new();
    let mut fields2: Punctuated<syn::Field, syn::token::Comma> =Punctuated::new();
    for field in fields.into_iter() {
        //Get attribute #[sql(field = __unknown__)]
        let mut attr=None;
        for a in get_attributes!(field, #[sql(field = __unknown__)]) {
            if attr.is_some() {
                anyhow::bail!("Only one #[sql(field = ...)] attribute is allowed per field!");
            }
            attr = Some(a);
        }
        if let Some(attr) = attr{
            //Parse the attribute
            let attr :FieldAttribute = syn::parse2(attr.clone())?;

            joined_fields.push(JoinedField{ field, table: attr.table, table_field: attr.table_field });
        }else{
            fields2.push(field);
        }
        
    }

    let fields=fields2;

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
    for driver in supported_drivers {
        result.add(sql_output_base(
            &item_name,
            &fields,
            joined_fields.clone(),
            &table,
            &driver.to_token_stream(),
        )?);
    }

    // panic!("{}", result);

    Ok(result.finalize().into())
}
