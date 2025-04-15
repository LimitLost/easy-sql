use convert_case::{Case, Casing};
use easy_macros::{
    anyhow::{self, Context},
    helpers::{context, parse_macro_input},
    macros::{always_context, get_attributes, has_attributes},
    quote::{quote, ToTokens},
    syn::{self, LitInt, LitStr},
};

use crate::{
    easy_lib_crate, easy_macros_helpers_crate, sql_crate,
    sql_derive::{sql_insert_base, sql_output_base, sql_update_base},
};

use super::ty_enum_value;

#[always_context]
pub fn sql_table(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;
    let item_name_tokens = item.ident.to_token_stream();

    let easy_macros_helpers_crate = easy_macros_helpers_crate();
    let sql_crate = sql_crate();
    let easy_lib_crate = easy_lib_crate();

    let fields = match item.fields {
        syn::Fields::Named(fields_named) => fields_named.named,
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
        let lit_str: LitStr = syn::parse2(attr_data).context("Invalid table name provided, expected string with  quotes")?;
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

    let mut is_primary_key = Vec::new();
    let mut is_unique = Vec::new();
    let mut primary_key_found = false;
    let mut field_types = Vec::new();
    let mut is_not_null = Vec::new();

    for field in fields.iter() {
        let (variant, is_field_not_null) = ty_enum_value(&field.ty)?;
        is_not_null.push(is_field_not_null);
        if let Some(variant) = variant {
            field_types.push(variant);
        } else {
            let current_is_primary_key = has_attributes!(field, #[sql(primary_key)]);
            if current_is_primary_key {
                if !primary_key_found {
                    primary_key_found = true;
                    is_primary_key.push(true);
                } else {
                    anyhow::bail!("Only one primary key is allowed per table!.");
                }
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
        }
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

    #[no_context]
    let table_version =
        table_version.with_context(context!("sql(version = x) attribute is required"))?;

    Ok(quote! {
        impl #sql_crate::DatabaseSetup for #item_name {
            async fn setup(
                conn: &mut (impl #sql_crate::EasyExecutor + Send + Sync),
            ) -> #easy_lib_crate::anyhow::Result<()> {
                use #easy_lib_crate::anyhow::Context;

                let table_exists = conn.query_setup(#sql_crate::TableExists{name: #table_name}).with_context(#easy_macros_helpers_crate::context!("Checking if table exists: {:?}".#table_name)).await?;

                if table_exists{
                    //TODO Get Table Version and migrate (alter table + update version) if neccessary
                }else{
                    use #sql_crate::EasyExecutor;
                    // Create table and create version in EasySqlTables
                    conn.query_setup(#sql_crate::CreateTable{table_name: #sql_crate::EasySqlTables::table_name(), fields: vec![
                        #(#sql_crate::TableField{name: #field_names_str, data_type: #sql_crate::SqlType::#field_types, is_primary_key: #is_primary_key, foreign_key: None, is_unique: #is_unique, is_not_null: #is_not_null},)*
                    ]}).await?;
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

        impl easy_lib::sql::SqlTable for #item_name {
            fn table_name() -> &'static str {
                #table_name
            }
        }

        impl easy_lib::sql::HasTable<#item_name> for #item_name{}

        #insert_impl
        #update_impl
        #output_impl
    }.into())
}
