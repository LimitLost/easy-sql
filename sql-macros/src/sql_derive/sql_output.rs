use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    syn::{self, parse::Parse, punctuated::Punctuated},
};

use easy_macros::{
    helpers::{context, parse_macro_input, TokensBuilder},
    macros::{always_context, get_attributes, has_attributes},
};
use sql_compilation_data::CompilationData;

use crate::{
    easy_lib_crate, sql_crate,  sql_macros_components::joined_field::JoinedField
};

#[always_context]
pub fn sql_output_base(
    item_name: &syn::Ident,
    fields: &Punctuated<syn::Field, syn::Token![,]>,
    joined_fields: Vec<JoinedField>,
    table: &TokenStream,
    driver: &TokenStream,
) -> anyhow::Result<TokenStream> {
    let field_names = fields.iter().map(|field| field.ident.as_ref().unwrap());
    let field_names_str = field_names.clone().map(|field| field.to_string());

    let joined_field_aliases=(0..joined_fields.len()).into_iter().map(|i|{
        format!("___easy_sql_joined_field_{}",i)
    }).collect::<Vec<_>>();

    let sql_crate = sql_crate();
    let easy_lib_crate = easy_lib_crate();

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
                let mut table_instance = #sql_crate::never::never_any::<#ref_table>();

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

    //Handle fields
    for field in
        fields.iter()
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
                    context!(#context_str),
                )?).with_context(
                    context!(#context_str2),
                )?,
            });
        }else{
            fields_quotes.push(quote! {
                #field_name: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #field_name_str).with_context(
                    context!(#context_str),
                )?,
            });
        }

    }


    let select_sqlx_str=fields.iter().map(|field|{
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
                <#ref_table as #sql_crate::SqlTable<#driver>>::table_name(),
            ));
        }
    });

    
        

    Ok(quote! {
        impl #sql_crate::SqlOutput<#table, #driver> for #item_name {
            type DataToConvert = #sql_crate::DriverRow<#driver>;

            fn sql_to_query<'a>(sql: #sql_crate::Sql, builder: #sql_crate::QueryBuilder<'a, #driver>) -> #easy_lib_crate::anyhow::Result<#sql_crate::QueryData<'a, #driver>> {
                #sql_crate::never::never_fn(|| {
                    //Check for validity
                    let table_instance = #sql_crate::never::never_any::<#table>();

                    //Joined fields check for validity
                    #(#joined_checks)*

                    Self {
                        #(#field_names: table_instance.#field_names,)*
                        //Joined fields
                        #(#joined_checks_field_names,)*
                    }
                });

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
                            table_name: Some(<#joined_checks_table_names as #sql_crate::SqlTable<#driver>>::table_name()),
                            name: #joined_checks_column_name_format.to_owned(),
                            alias: Some(#joined_field_aliases.to_owned()),
                        },
                    )*
                ];

                sql.query_output(builder, requested_columns)
            }
            
            fn select_sqlx(current_query: &mut String) {
                let delimeter = <#driver as #sql_crate::Driver>::identifier_delimiter();
                #select_sqlx_str_call
                #(#select_sqlx_joined)*
            }

            fn convert(data: #sql_crate::DriverRow<#driver>) -> ::anyhow::Result<Self> {
                use ::anyhow::Context;
                use #sql_crate::macro_support::context;

                Ok(Self {
                    #(
                        #fields_quotes
                    )*
                    #(
                        #joined_checks_field_names: <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&data, #joined_field_aliases).with_context(
                            context!(#context_strs2),
                        )?,
                    )*
                })
            }
        }

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


#[always_context]
pub fn sql_output(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
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
