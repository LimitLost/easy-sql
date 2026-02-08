use std::collections::HashMap;

use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn::{self, LitStr},
};
use convert_case::{Case, Casing};
use easy_macros::{
    TokensBuilder, always_context, get_attributes, has_attributes, parse_macro_input,
};
use easy_sql_compilation_data::CompilationData;
#[cfg(feature = "migrations")]
use {easy_macros::context, easy_sql_compilation_data::TableDataVersion, syn::LitInt};

use crate::{
    derive::{sql_insert_base, sql_output_base, sql_update_base},
    macros_components::joined_field::JoinedField,
    sql_crate,
};

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
pub fn table(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let item = parse_macro_input!(item as syn::ItemStruct);
    let item_name = &item.ident;
    let item_name_tokens = item.ident.to_token_stream();

    let sql_crate = sql_crate();
    let macro_support = quote! { #sql_crate::macro_support };

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
    #[cfg(feature = "check_duplicate_table_names")]
    let mut table_name_attr_used = false;

    //Use name provided by the user if it exists
    if let Some(attr_data) = get_attributes!(item, #[sql(table_name = __unknown__)])
        .into_iter()
        .next()
    {
        let lit_str: LitStr = syn::parse2(attr_data.clone())
            .context("Invalid table name provided, expected string with  quotes")?;
        table_name = lit_str.value();
        #[cfg(feature = "check_duplicate_table_names")]
        {
            table_name_attr_used = true;
        }
    }
    #[cfg(feature = "migrations")]
    let no_version = has_attributes!(item, #[sql(no_version)]);

    #[cfg(feature = "migrations")]
    let mut version_test: Option<LitInt> = None;

    #[cfg(feature = "migrations")]
    for attr_data in get_attributes!(item, #[sql(version_test = __unknown__)]) {
        if version_test.is_some() {
            anyhow::bail!("Only one version_test attribute is allowed");
        }
        let parsed: LitInt =
            syn::parse2(attr_data.clone()).context("Expected version_test to be an integer")?;
        version_test = Some(parsed);
    }

    #[cfg(not(feature = "migrations"))]
    if !get_attributes!(item, #[sql(version = __unknown__)]).is_empty() {
        anyhow::bail!(
            "The #[sql(version = ...)] attribute requires the `migrations` feature to be enabled."
        );
    }

    // Determine if migrations should be skipped
    #[cfg(not(feature = "migrations"))]
    let skip_migrations = true;

    #[cfg(feature = "migrations")]
    let skip_migrations = no_version;

    let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;

    #[cfg(feature = "check_duplicate_table_names")]
    if let Some(entries) = compilation_data.used_table_names.get(&table_name) {
        if entries.len() > 1 {
            let mut lines = entries
                .iter()
                .map(|entry| format!("- {} (file: {})", entry.struct_name, entry.filename))
                .collect::<Vec<_>>();
            lines.sort();

            if table_name_attr_used {
                anyhow::bail!(
                    "Multiple tables use the same table name `{}`.\n\
Each table name must be unique.\n\
Found in:\n{}\n\
Tip: change `#[sql(table_name = ...)]`",
                    table_name,
                    lines.join("\n")
                );
            } else {
                anyhow::bail!(
                    "Multiple tables use the same table name `{}`.\n\
Each table name must be unique.\n\
Found in:\n{}\n\
Tip: Use `#[sql(table_name = ...)]` or rename one of the structs",
                    table_name,
                    lines.join("\n")
                );
            }
        }
    }

    let supported_drivers = super::supported_drivers(&item, &compilation_data, false)?;

    #[cfg(feature = "migrations")]
    let mut table_version = None;

    #[cfg(feature = "migrations")]
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

    #[cfg(feature = "migrations")]
    if no_version && (table_version.is_some() || version_test.is_some()) {
        return Err(syn::Error::new_spanned(
            &item.ident,
            "#[sql(no_version)] and #[sql(version = ...)] are mutually exclusive. \
             Use #[sql(no_version)] to disable migrations, or #[sql(version = ...)] to enable them, but not both."
        ).into());
    }

    #[cfg(feature = "migrations")]
    if version_test.is_some() && table_version.is_some() {
        return Err(syn::Error::new_spanned(
            &item.ident,
            "#[sql(version_test = ...)] replaces #[sql(version = ...)] and they cannot be used together."
        ).into());
    }

    #[cfg(feature = "migrations")]
    let (table_version_i64, migrations, unique_id) = if skip_migrations {
        (0i64, quote! { Vec::new() }, quote! { "" })
    } else {
        if let Some(version_test) = &version_test {
            let test_version = version_test
                .base10_parse::<i64>()
                .context("Expected base10 int for version_test")?;
            table_version = Some(test_version);
        }

        #[no_context_inputs]
        let table_version =
            table_version.with_context(context!("Either #[sql(version = x)] (enable migrations) or #[sql(no_version)] (skip migrations) attribute is required"))?;

        //Sqlite doesn't support unsigned integers, so we need to do this
        let table_version_i64 = table_version as i64;

        let unique_id_attr = get_attributes!(item, #[sql(unique_id = __unknown__)])
            .into_iter()
            .next();

        if version_test.is_some() && unique_id_attr.is_none() {
            anyhow::bail!(
                "#[sql(unique_id = ...)] is required when using #[sql(version_test = ...)]"
            );
        }

        let unique_id_attr = unique_id_attr.context("Sql build macro is required (reload VS Code or save if unique id is already generated)")?;
        let unique_id_lit: LitStr =
            syn::parse2(unique_id_attr.clone()).context("Unique id should be string")?;

        let converted_to_version = TableDataVersion::from_struct(&item, table_name.clone())?;

        let migrations = if let Some(table_data) =
            compilation_data.tables.get(&unique_id_lit.value())
        {
            let migrations = compilation_data.generate_migrations(
                &unique_id_lit.value(),
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
                unique_id_lit.value(),
                CompilationData::data_location()?.display()
            );
        };

        let unique_id = unique_id_lit.to_token_stream();
        (table_version_i64, migrations, unique_id)
    };

    #[cfg(not(feature = "migrations"))]
    let (table_version_i64, migrations, unique_id) = (0i64, quote! { Vec::new() }, quote! { "" });

    let mut result_builder = TokensBuilder::default();

    let output_impl = sql_output_base(
        item_name,
        &fields,
        Vec::<JoinedField>::new(),
        &item_name_tokens,
        &supported_drivers,
    )?;
    result_builder.add(output_impl);

    let insert_impl = sql_insert_base(
        item_name,
        &fields,
        &item_name_tokens,
        &supported_drivers,
        vec![] as Vec<syn::Ident>,
    )?;
    result_builder.add(insert_impl);
    let update_impl = sql_update_base(item_name, &fields, &item_name_tokens, &supported_drivers)?;
    result_builder.add(update_impl);

    let mut primary_keys = Vec::new();

    for field in fields.iter() {
        //Primary Key Check
        if has_attributes!(field, #[sql(primary_key)]) {
            primary_keys.push(field.ident.as_ref()?.to_string());
        }
    }

    for driver in supported_drivers {
        let mut foreign_keys = HashMap::new();

        //TODO Primary key types check (compile time in the build script)
        // let mut primary_key_types=Vec::new();

        let mut is_unique = Vec::new();
        let mut field_types = Vec::new();
        let mut is_not_null = Vec::new();
        let mut is_auto_increment_list = Vec::new();
        // First token streamn represents data before the driver
        // Second token stream represents data after the driver
        let mut default_values: Vec<TokenStream> = Vec::new();

        for field in fields.iter() {
            let field_type = &field.ty;

            //Auto Increment Check
            if has_attributes!(field, #[sql(auto_increment)]) {
                is_auto_increment_list.push(true);
            } else {
                is_auto_increment_list.push(false);
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
            //Get `Is Not Null`
            let is_field_not_null = match &field_type {
                syn::Type::Path(type_path) => {
                    if let Some(last_segment) = type_path.path.segments.last() {
                        last_segment.ident != "Option"
                    } else {
                        true
                    }
                }
                _ => true,
            };
            is_not_null.push(is_field_not_null);

            //Unique Check
            is_unique.push(has_attributes!(field, #[sql(unique)]));

            //Binary Check and get field type
            let binary_field = if has_attributes!(field, #[sql(bytes)]) {
                field_types.push(quote! {
                    {
                        #macro_support::TypeInfo::name(
                            &<Vec<u8> as #macro_support::Type<#macro_support::InternalDriver<#driver>>>::type_info(),
                        )
                        .to_owned()
                    }
                });
                true
            } else {
                field_types.push(quote! {
                    {
                        #macro_support::TypeInfo::name(
                            &<#field_type as #macro_support::Type<#macro_support::InternalDriver<#driver>>>::type_info(),
                        )
                        .to_owned()
                    }
                });

                false
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

                    //Convert provided default value to bytes
                    default_values.push(quote! {
                        {
                            //Check if default value has valid type for the current column
                            let _ = ||{
                                let mut table_instance = #macro_support::never_any::<#item_name>();
                                table_instance.#field_name = #default_value;
                            };

                            let default_v = #macro_support::Context::context(
                                #macro_support::to_binary(#default_value),
                                #error_context,
                            )?;

                            Some(#sql_crate::ToDefault::to_default(default_v))
                        }
                    });
                } else {
                    default_values.push(quote! {
                        {
                            //Check if default value has valid type for the current column
                            let _ = ||{
                                let mut table_instance = #macro_support::never_any::<#item_name>();
                                table_instance.#field_name = #default_value;
                            };

                            Some(#sql_crate::ToDefault::to_default(#default_value))
                        }
                    });
                }

                default_value_found = true;
            }
            if !default_value_found {
                default_values.push(quote! {None});
            }
        }

        let primary_key_check = if primary_keys.is_empty() {
            quote! {
                let _ = || {
                    fn __easy_sql_assert<T: #sql_crate::markers::AllowsNoPrimaryKey>() {}
                    __easy_sql_assert::<#driver>();
                };
            }
        } else {
            quote! {}
        };

        let auto_increment_pk_check = if primary_keys.len() != 1
            && is_auto_increment_list.iter().any(|v| *v)
        {
            quote! {
                let _ = || {
                    fn __easy_sql_assert<T: #sql_crate::markers::SupportsAutoIncrementCompositePrimaryKey>() {}
                    __easy_sql_assert::<#driver>();
                };
            }
        } else {
            quote! {}
        };

        let multi_auto_increment_check = if is_auto_increment_list.iter().filter(|v| **v).count()
            > 1
        {
            quote! {
                let _ = || {
                    fn __easy_sql_assert<T: #sql_crate::markers::SupportsMultipleAutoIncrementColumns>() {}
                    __easy_sql_assert::<#driver>();
                };
            }
        } else {
            quote! {}
        };

        // Foreign keys converted
        let foreign_keys = {
            let mut foreign_keys_converted = Vec::new();
            for (foreign_table, (referenced_fields, cascade)) in foreign_keys.iter() {
                foreign_keys_converted.push(quote! {
                    (
                        <#foreign_table as #sql_crate::Table<#driver>>::table_name(),
                        (
                            vec![#(#referenced_fields),*],
                            <#foreign_table as #sql_crate::Table<#driver>>::primary_keys(),
                            #cascade
                        ),
                    )
                });
            }
            foreign_keys_converted
        };

        let create_table = quote! {
            <#driver as #sql_crate::Driver>::create_table(
                    conn,
                    #table_name,
                    vec![
                        #(
                        #sql_crate::driver::TableField{
                            name: #field_names_str,
                            data_type: #field_types,
                            is_unique: #is_unique,
                            is_not_null: #is_not_null,
                            default: #default_values,
                            is_auto_increment: #is_auto_increment_list,
                        },
                        )*
                    ],
                    vec![#(#primary_keys),*],
                    {
                        vec![#(#foreign_keys),*]
                        .into_iter()
                        .collect()
                    },
                ).await?;
        };

        let setup_body = if skip_migrations {
            // Without migrations, just create the table without version tracking
            quote! {
                #primary_key_check
                #auto_increment_pk_check
                #multi_auto_increment_check
                #create_table
                Ok(())
            }
        } else {
            // With migrations, use version tracking and migrations
            quote! {
                type _EasySqlMigrationDriver = #driver;

                let current_version_number = #sql_crate::EasySqlTables_get_version!(#driver, *conn,#unique_id);

                if let Some(current_version_number) = current_version_number{
                    #migrations
                }else{
                    // Create table and create version in EasySqlTables
                    #primary_key_check
                    #auto_increment_pk_check
                    #multi_auto_increment_check
                    #create_table
                    #sql_crate::EasySqlTables_create!(#driver, *conn, #unique_id.to_string(), #table_version_i64);
                }

                Ok(())
            }
        };

        result_builder.add(quote! {
            impl #sql_crate::DatabaseSetup<#driver> for #item_name {

                async fn setup(
                    conn: &mut (impl #sql_crate::EasyExecutor<#driver> + Send + Sync),
                ) -> #macro_support::Result<()> {
                    #setup_body
                }
            }

        });
    }

    result_builder.add(quote! {
        impl #sql_crate::markers::HasTable<#item_name> for #item_name{}

        impl #sql_crate::markers::NotJoinedTable for #item_name {}

        impl<EasySqlD:#sql_crate::Driver> #sql_crate::Table<EasySqlD> for #item_name {

            fn table_name() -> &'static str {
                #table_name
            }

            fn primary_keys() -> Vec<&'static str>{
                vec![#(#primary_keys),*]
            }


            #[inline(always)]
            fn table_joins(current_query: &mut String) {

            }
        }
    });

    //panic!("{}", result);

    Ok(result_builder.finalize().into())
}
