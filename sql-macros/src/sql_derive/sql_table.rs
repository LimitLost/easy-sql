use std::collections::HashMap;

use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, LitInt, LitStr},
};
use convert_case::{Case, Casing};
use easy_macros::{
    helpers::{TokensBuilder, context, parse_macro_input},
    macros::{always_context, get_attributes, has_attributes},
};
use sql_compilation_data::{CompilationData, TableDataVersion};

use crate::{
    sql_crate,
    sql_derive::{sql_insert_base, sql_output_base, sql_update_base},
    sql_macros_components::joined_field::JoinedField,
};

use super::ty_enum_value;

mod keywords {
    syn::custom_keyword!(cascade);
}

struct ForeignKeyParsed {
    table_struct: syn::Path,
    cascade: bool,
}

#[always_context]
impl syn::parse::Parse for ForeignKeyParsed {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let table_struct = input.parse()?;
        let cascade = if input.is_empty() {
            false
        } else {
            input.parse::<syn::Token![,]>()?;
            input.parse::<keywords::cascade>()?;
            true
        };
        Ok(ForeignKeyParsed {
            table_struct,
            cascade,
        })
    }
}

#[always_context]
pub fn sql_table(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;
    let item_name_tokens = item.ident.to_token_stream();

    let sql_crate = sql_crate();

    let fields = match &item.fields {
        syn::Fields::Named(fields_named) => fields_named.named.clone(),
        syn::Fields::Unnamed(_) => {
            anyhow::bail!("Unnamed struct fields are not supported")
        }
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };
    let field_names_str = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string())
        .collect::<Vec<_>>();

    let mut table_name = item_name.to_string().to_case(Case::Snake);

    //Use name provided by the user if it exists
    if let Some(attr_data) = get_attributes!(item, #[sql(table_name = __unknown__)])
        .into_iter()
        .next()
    {
        let lit_str: LitStr = syn::parse2(attr_data.clone())
            .context("Invalid table name provided, expected string with  quotes")?;
        table_name = lit_str.value();
    }

    let mut primary_keys = Vec::new();
    let mut foreign_keys = HashMap::new();

    let mut auto_increment = false;

    //TODO Primary key types check (compile time in the build script)
    // let mut primary_key_types=Vec::new();

    let mut is_unique = Vec::new();
    let mut field_types = Vec::new();
    let mut is_not_null = Vec::new();
    // First token streamn represents data before the driver
    // Second token stream represents data after the driver
    let mut default_values: Vec<Box<dyn Fn(&TokenStream) -> TokenStream>> = Vec::new();

    for field in fields.iter() {
        //Auto Increment Check
        if has_attributes!(field, #[sql(auto_increment)]) {
            if auto_increment {
                anyhow::bail!("Auto increment is only supported for single primary key");
            }
            auto_increment = true;
        }

        //Foreign Key Check
        for foreign_key in get_attributes!(field, #[sql(foreign_key = __unknown__)]) {
            let foreign_key: ForeignKeyParsed = syn::parse2(foreign_key.clone())
                .context("Expected foreign key to be a table name")?;

            let fields: &mut (Vec<String>, bool) = foreign_keys
                .entry(foreign_key.table_struct)
                .or_insert((Default::default(), foreign_key.cascade));
            fields.0.push(field.ident.as_ref()?.to_string());
            if foreign_key.cascade {
                fields.1 = true;
            }
        }
        //Get `Type Enum Variant` and `Is Not Null`
        let (variant, is_field_not_null) = ty_enum_value(&field.ty, &sql_crate)?;
        is_not_null.push(is_field_not_null);

        //Primary Key Check
        if has_attributes!(field, #[sql(primary_key)]) {
            primary_keys.push(field.ident.as_ref()?.to_string());
        }
        //Unique Check
        is_unique.push(has_attributes!(field, #[sql(unique)]));

        //Binary Check (Binary if field is not supported)
        let binary_field = if let Some(variant) = variant {
            //Field is supported
            field_types.push(variant);
            false
        } else {
            //Field is not supported
            if has_attributes!(field, #[sql(bytes)]) {
                field_types.push(quote! {Bytes});
                true
            } else {
                anyhow::bail!(
                    "Field type {:?} is not supported, use #[sql(bytes)] to convert it into bytes",
                    field.ty
                );
            }
        };

        //Default Value Check
        let mut default_value_found = false;
        for default_value in get_attributes!(field, #[sql(default = __unknown__)]) {
            let field_name = field.ident.as_ref()?;

            if default_value_found {
                anyhow::bail!("Only one default value is allowed");
            }
            //Every default value should be an expression
            syn::parse2::<syn::Expr>(default_value.clone())
                .context("Expected default value to be an expression")?;

            if binary_field {
                let error_context = format!(
                    "Converting default value `{}` to bytes for field `{}`, struct name: `{}`, table name: `{}`",
                    default_value.to_token_stream(),
                    field_name,
                    item_name,
                    table_name
                );

                let before_lazy_static = quote! {
                        //Test if default value to_binary will be successful
                        //Even in release mode, just in case, it's low cost anyway
                        #sql_crate::to_binary(#default_value).context(#error_context)?;
                };
                let after_lazy_static = quote! {
                            //Check if default value has valid type for the current column
                            #sql_crate::never::never_fn(||{
                                let mut table_instance = #sql_crate::never::never_any::<#item_name>();
                                table_instance.#field_name = #default_value;
                            });

                            Some(&*DEFAULT_VALUE)
                };

                let sql_crate = sql_crate.clone();

                //Convert provided default value to bytes
                default_values.push(Box::new(move |d|{
                    quote! {
                        {
                            #before_lazy_static

                            #sql_crate::lazy_static!{
                                static ref DEFAULT_VALUE: <#d as #sql_crate::Driver>::Value<'static> = #sql_crate::to_binary(#default_value).unwrap().into();
                            }

                            #after_lazy_static
                        }
                    }}));
            } else {
                let after_lazy_static = quote! {
                            //Check if default value has valid type for the current column
                            #sql_crate::never::never_fn(||{
                                let mut table_instance = #sql_crate::never::never_any::<#item_name>();
                                table_instance.#field_name = #default_value;
                            });

                            Some(&*DEFAULT_VALUE)
                };

                let sql_crate = sql_crate.clone();

                default_values.push(Box::new(move|d|quote! {
                        {
                            #sql_crate::lazy_static!{
                                static ref DEFAULT_VALUE: <#d as #sql_crate::Driver>::Value<'static> = (#default_value).into();
                            }

                            #after_lazy_static
                        }
                    }));
            }

            default_value_found = true;
        }
        if !default_value_found {
            default_values.push(Box::new(|_| quote! {None}));
        }
    }

    if primary_keys.is_empty() {
        anyhow::bail!(
            "No primary key found, please add #[sql(primary_key)] to one of the fields (Sqlite always has one)"
        );
    }

    if primary_keys.len() != 1 && auto_increment {
        anyhow::bail!(
            "Auto increment is only supported for single primary key (Sqlite contrains this limitation)"
        );
    }

    let mut table_version = None;

    for version in get_attributes!(item, #[sql(version = __unknown__)]) {
        if table_version.is_some() {
            anyhow::bail!("Only one version attribute is allowed");
        }
        let version: LitInt = syn::parse2(version.clone())
            .context("Expected literal int in the sql(version) attribute")?;
        let n: i64 = version
            .base10_parse()
            .context("Expected base10 literal int in the sql(version) attribute")?;
        table_version = Some(n);
    }

    #[no_context_inputs]
    let table_version =
        table_version.with_context(context!("#[sql(version = x)] attribute is required"))?;

    //Sqlite doesn't support unsigned integers, so we need to do this
    let table_version_i64 = table_version as i64;

    let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;

    let supported_drivers = super::supported_drivers(&item, &compilation_data)?;

    let unique_id=get_attributes!(item, #[sql(unique_id = __unknown__)]).into_iter().next().context("Sql build macro is required (reload VS Code or save if unique id is already generated)")?;
    let unique_id: LitStr = syn::parse2(unique_id.clone()).context("Unique id should be string")?;

    let converted_to_version = TableDataVersion::from_struct(&item, table_name.clone())?;

    //Generate migrations
    //Check if old version is the same, show error otherwise

    let migrations = if let Some(table_data) = compilation_data.tables.get(&unique_id.value()) {
        let migrations = compilation_data.generate_migrations(
            &unique_id.value(),
            &converted_to_version,
            table_version,
            &sql_crate,
            &item_name.to_token_stream(),
        )?;

        if let Some(this_version) = table_data.saved_versions.get(&table_version)
            && this_version != &converted_to_version
        {
            return Err(anyhow::anyhow!(
                    "When you're done with making changes to the table, don't forget to update the version number in the #[sql(version = x)] attribute!"
                )).with_context(context!("table in easy_sql.ron: {:?}\r\n\r\nnew table structure: {:?}",this_version,converted_to_version));
        }

        migrations
    } else {
        anyhow::bail!(
            "Table with unique id {} not found in the compilation data (try to save the file)\r\n=====\r\nDEBUG: Compilation data expected location: `{}`",
            unique_id.value(),
            CompilationData::data_location()?.display()
        );
    };

    let mut result_builder = TokensBuilder::default();
    for driver in supported_drivers {
        let driver_tokens = driver.to_token_stream();

        let insert_impl = sql_insert_base(
            item_name,
            &fields,
            &item_name_tokens,
            &driver_tokens,
            vec![] as Vec<syn::Ident>,
        )?;
        let update_impl =
            sql_update_base(item_name, &fields, &item_name_tokens, &driver_tokens, true)?;
        let output_impl = sql_output_base(
            item_name,
            &fields,
            Vec::<JoinedField>::new(),
            &item_name_tokens,
            &driver_tokens,
        )?;

        let default_values: Vec<TokenStream> =
            default_values.iter().map(|f| f(&driver_tokens)).collect();

        // Foreign keys converted
        let foreign_keys = {
            let mut foreign_keys_converted = Vec::new();
            for (foreign_table, (referenced_fields, cascade)) in foreign_keys.iter() {
                foreign_keys_converted.push(quote! {
                    (
                        <#foreign_table as #sql_crate::SqlTable<#driver>>::table_name(),
                        (
                            vec![#(#referenced_fields),*],
                            <#foreign_table as #sql_crate::SqlTable<#driver>>::primary_keys(),
                            #cascade
                        ),
                    )
                });
            }
            foreign_keys_converted
        };

        result_builder.add(quote! {
        impl #sql_crate::DatabaseSetup<#driver> for #item_name {

            async fn setup(
                conn: &mut (impl #sql_crate::EasyExecutor<#driver> + Send + Sync),
            ) -> ::anyhow::Result<()> {
                use ::anyhow::Context;

                let current_version_number = #sql_crate::EasySqlTables_get_version!(#driver, conn,#unique_id);

                if let Some(current_version_number) = current_version_number{
                    use #sql_crate::EasyExecutor;

                    #migrations
                }else{
                    // Create table and create version in EasySqlTables
                    <#driver as #sql_crate::Driver>::create_table(
                        conn,
                        #table_name,
                        vec![
                            #(
                            #sql_crate::TableField{
                                name: #field_names_str,
                                data_type: #sql_crate::SqlType::#field_types,
                                is_unique: #is_unique,
                                is_not_null: #is_not_null,
                                default: #default_values,
                            },
                            )*
                        ],
                        vec![#(#primary_keys),*],
                        #auto_increment,
                        {
                            vec![#(#foreign_keys),*]
                            .into_iter()
                            .collect()
                        },
                    ).await?;
                    #sql_crate::EasySqlTables_create!(#driver, conn, #unique_id.to_string(), #table_version_i64);
                }

                Ok(())
            }
        }

        impl #sql_crate::SqlTable<#driver> for #item_name {

            fn table_name() -> &'static str {
                #table_name
            }

            fn primary_keys() -> Vec<&'static str>{
                vec![#(#primary_keys),*]
            }

            fn table_joins() -> Vec<#sql_crate::TableJoin<'static, #driver>> {
                vec![]
            }
        }

        }    );

        result_builder.add(insert_impl);
        result_builder.add(update_impl);
        result_builder.add(output_impl);
    }

    result_builder.add(quote! {
        impl #sql_crate::HasTable<#item_name> for #item_name{}
    });

    //panic!("{}", result);

    Ok(result_builder.finalize().into())
}
