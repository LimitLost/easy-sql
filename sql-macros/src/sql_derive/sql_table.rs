use std::collections::HashMap;

use convert_case::{Case, Casing};
use easy_macros::{
    anyhow::{self, Context},
    helpers::{context, parse_macro_input},
    macros::{always_context, get_attributes, has_attributes},
    quote::{quote, ToTokens},
    syn::{self, LitInt, LitStr},
};
use sql_compilation_data::{CompilationData, TableDataVersion};

use crate::{
    easy_lib_crate, easy_macros_helpers_crate, sql_crate,
    sql_derive::{sql_insert_base, sql_output_base, sql_update_base},
};

use super::ty_enum_value;

mod keywords{
    use easy_macros::syn;
    syn::custom_keyword!(cascade);
}

struct ForeignKeyParsed{
    table_struct:syn::Path,
    cascade:bool,
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
        Ok(ForeignKeyParsed { table_struct, cascade })
    }
}

#[always_context]
pub fn sql_table(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;
    let item_name_tokens = item.ident.to_token_stream();

    let easy_macros_helpers_crate = easy_macros_helpers_crate();
    let sql_crate = sql_crate();
    let easy_lib_crate = easy_lib_crate();

    let fields = match &item.fields {
        syn::Fields::Named(fields_named) => fields_named.named.clone(),
        syn::Fields::Unnamed(_) => {
            anyhow::bail!("Unnamed struct fields are not supported")
        }
        syn::Fields::Unit => anyhow::bail!("Unit struct is not supported"),
    };
    let field_names_str = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().to_string());

    let mut table_name = item_name.to_string().to_case(Case::Snake);

    //Use name provided by the user if it exists
    for attr_data in get_attributes!(item, #[sql(table_name = "__unknown__")]) {
        let lit_str: LitStr = syn::parse2(attr_data.clone()).context("Invalid table name provided, expected string with  quotes")?;
        table_name = lit_str.value();
        break;
    }
        

    let insert_impl = sql_insert_base(
        &item_name,
        &fields,
        &item_name_tokens,
        vec![] as Vec<syn::Ident>,
    )?;
    let update_impl = sql_update_base(&item_name, &fields, &item_name_tokens)?;
    let output_impl = sql_output_base(&item_name, &fields, &item_name_tokens);

    let mut primary_keys=Vec::new();
    let mut foreign_keys=HashMap::new();

    let mut auto_increment=false;

    //TODO Primary key types check (compile time in the build script)
    // let mut primary_key_types=Vec::new();

    let mut is_primary_key = Vec::new();
    let mut is_unique = Vec::new();
    let mut field_types = Vec::new();
    let mut is_not_null = Vec::new();

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

            let fields: &mut (Vec<String>,bool) = foreign_keys
                .entry(foreign_key.table_struct)
                .or_insert((Default::default(),foreign_key.cascade));
            fields.0.push(field.ident.as_ref()?.to_string());
            if foreign_key.cascade {
                fields.1=true;
            }
        }

        let (variant, is_field_not_null) = ty_enum_value(&field.ty)?;
        is_not_null.push(is_field_not_null);
        if let Some(variant) = variant {
            field_types.push(variant);
        } else {
            let current_is_primary_key = has_attributes!(field, #[sql(primary_key)]);
            if current_is_primary_key {
                    primary_keys.push(field.ident.as_ref()?.to_string());
                    is_primary_key.push(true);
            } else {
                is_primary_key.push(false);
            }
            is_unique.push(has_attributes!(field, #[sql(unique)]));

            if has_attributes!(field, #[sql(bytes)]) {
                field_types.push(quote! {Bytes});
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
            if default_value_found {
                anyhow::bail!("Only one default value is allowed");
            }
            let default_value: syn::Expr = syn::parse2(default_value.clone())
                .context("Expected default value to be an expression")?;

            if binary_field {
                let error_context = format!(
                    "Converting default value `{}` to bytes for field `{}`, struct name: `{}`, table name: `{}`",
                    default_value.to_token_stream(),
                    field.ident.as_ref().unwrap(),
                    item_name,
                    table_name
                );

                //Convert provided default value to bytes
                default_values.push(quote! {
                        {
                            //Test if default value to_bytes will be successful
                            //Even in release mode, just in case, it's low cost anyway
                            #sql_crate::to_bytes(#default_value).context(#error_context)?;

                            #sql_crate::lazy_static!{
                                static ref DEFAULT_VALUE: #sql_crate::SqlValueMaybeRef<'static> = #sql_crate::to_bytes(#default_value).unwrap().into();
                            }

                            Some(&*DEFAULT_VALUE)
                        }
                    });
            } else {
                default_values.push(quote! {
                        {
                            #sql_crate::lazy_static!{
                                static ref DEFAULT_VALUE: #sql_crate::SqlValueMaybeRef<'static> = (#default_value).into();
                            }

                            Some(&*DEFAULT_VALUE)
                        }
                    });
            }

            default_value_found = true;
        }
        if !default_value_found {
            default_values.push(quote! {None});
        }
    }
    }

    if primary_keys.len() !=1 && auto_increment{
        anyhow::bail!("Auto increment is only supported for single primary key (Sqlite contrains this limitation)");
    }

    let mut table_version = None;

    for version in get_attributes!(item, #[sql(version = __unknown__)]) {
        if table_version.is_some() {
            anyhow::bail!("Only one version attribute is allowed");
        }
        let version: LitInt = syn::parse2(version.clone())
            .context("Expected literal int in the sql(version) attribute")?;
        let n: u64 = version
            .base10_parse()
            .context("Expected base10 literal int in the sql(version) attribute")?;
        table_version = Some(n);
    }

    // Foreign keys converted
    let foreign_keys={
        let mut foreign_keys_converted=Vec::new();
        for (foreign_table, (referenced_fields,cascade)) in foreign_keys {
            foreign_keys_converted.push(quote! {
                (
                    <#foreign_table as #sql_crate::SqlTable>::table_name(),
                    (
                        vec![#(#referenced_fields),*],
                        <#foreign_table as #sql_crate::SqlTable>::primary_keys(),
                        #cascade
                    ),
                )
            });
        }
        foreign_keys_converted
    };

    #[no_context_inputs]
    let table_version =
        table_version.with_context(context!("#[sql(version = x)] attribute is required"))?;

    let compilation_data=CompilationData::load()?;

    let unique_id=get_attributes!(item, #[sql(unique_id = __unknown__)]).into_iter().next().context("Sql build macro is required (reload VS Code or save if unique id is already generated)")?;
    let unique_id: LitStr = syn::parse2(unique_id.clone()).context("Unique id should be string")?;

    let converted_to_version=TableDataVersion::from_struct(&item, table_name.clone())?;

    let migrations=compilation_data.generate_migrations(&unique_id.value(), &converted_to_version, table_version,&sql_crate)?;

    Ok(quote! {
        impl #sql_crate::DatabaseSetup for #item_name {

            async fn setup(
                conn: &mut (impl #sql_crate::EasyExecutor + Send + Sync),
            ) -> #easy_lib_crate::anyhow::Result<()> {
                use #easy_lib_crate::anyhow::Context;

                let table_exists = conn.query_setup(#sql_crate::TableExists{name: #table_name}).await.with_context(#easy_macros_helpers_crate::context!("Checking if table exists: {:?}",#table_name))?;

                if table_exists{
                    use #sql_crate::EasyExecutor;
                    let current_version_number = #sql_crate::EasySqlTables::get_version(conn,#unique_id).await?;
                    #migrations
                }else{
                    use #sql_crate::EasyExecutor;
                    // Create table and create version in EasySqlTables
                    conn.query_setup(#sql_crate::CreateTable{
                        table_name: #table_name, 
                        fields: vec![
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
                        auto_increment: #auto_increment,
                        primary_keys: vec![#(#primary_keys),*],
                        foreign_keys: {
                            vec![#(#foreign_keys),*]
                            .into_iter()
                            .collect()
                        },
                    }).await?;
                    #sql_crate::EasySqlTables::create(conn, #table_name.to_string(), #table_version).await?;
                }

                //If table doesn't exist ( https://stackoverflow.com/questions/1601151/how-do-i-check-in-sqlite-whether-a-table-exists )
                //Save current version to easy_sql_tables table

                //Create Unique id for every table (saved inside of macro file before compilation)
                //Every Unique id is generated by the build.rs
                

                //Create migrations based on table number change
                //Check if default value was provided by user (if field is not Option<>)

                //TODO In procedural macro
                // save table fields into a file (when the version number attribute changed)

                //In sqlite you can only add new columns and rename old ones
                //(without recreating a table https://www.sqlitetutorial.net/sqlite-alter-table/ )

                Ok(())
            }
        }

        impl #sql_crate::SqlTable for #item_name {
            fn table_name() -> &'static str {
                #table_name
            }

            fn primary_keys() -> Vec<&'static str>{
                vec![#(#primary_keys),*]
            }

            fn table_joins() -> Vec<TableJoin> {
                vec![]
            }
        }

        impl #sql_crate::HasTable<#item_name> for #item_name{}

        #insert_impl
        #update_impl
        #output_impl
    }.into())
}
