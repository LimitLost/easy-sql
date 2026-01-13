use anyhow::Context;
use easy_macros::always_context;
use easy_macros::{TokensBuilder, parse_macro_input};
use quote::ToTokens;
use quote::quote;
use sql_compilation_data::CompilationData;
use syn::Path;
use syn::punctuated::Punctuated;
use syn::{self, parse::Parse};

use crate::macros_components::expr::Expr;
use crate::query_macro_components::ProvidedDrivers;
use crate::sql_crate;

use crate::macros_components::keyword;

struct Input {
    drivers: Option<Punctuated<syn::Path, syn::Token![,]>>,
    struct_name: syn::Ident,
    main_table: syn::Path,
    joins: Vec<Join>,
}
enum Join {
    Inner { table: syn::Path, on: Expr },
    Left { table: syn::Path, on: Expr },
    Right { table: syn::Path, on: Expr },
    Cross { table: syn::Path },
}

#[always_context]
impl Parse for Join {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(keyword::inner) {
            input.parse::<keyword::inner>()?;
            input.parse::<keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<keyword::on>()?;

            let on = input.parse::<Expr>()?;
            Ok(Join::Inner { table, on })
        } else if lookahead.peek(keyword::left) {
            input.parse::<keyword::left>()?;
            input.parse::<keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<keyword::on>()?;

            let on = input.parse::<Expr>()?;
            Ok(Join::Left { table, on })
        } else if lookahead.peek(keyword::right) {
            input.parse::<keyword::right>()?;
            input.parse::<keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            input.parse::<keyword::on>()?;

            let on = input.parse::<Expr>()?;
            Ok(Join::Right { table, on })
        } else if lookahead.peek(keyword::cross) {
            input.parse::<keyword::cross>()?;
            input.parse::<keyword::join>()?;

            let table = input.parse::<syn::Path>()?;

            Ok(Join::Cross { table })
        } else {
            Err(input.error("Expected join type"))
        }
    }
}

#[always_context]
impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut drivers = None;
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);
            drivers = Some(content.parse_terminated(syn::Path::parse, syn::Token![,])?);
        }

        let struct_name = input.parse::<syn::Ident>()?;
        // Separator
        input.parse::<syn::Token![|]>()?;
        let main_table = input.parse::<syn::Path>()?;
        let mut joins = Vec::new();

        while !input.is_empty() {
            let join = input.parse::<Join>()?;
            joins.push(join);
        }

        Ok(Input {
            drivers,
            struct_name,
            main_table,
            joins,
        })
    }
}

#[always_context]
fn supported_drivers(
    current_drivers: Option<Punctuated<syn::Path, syn::Token![,]>>,
    compilation_data: &CompilationData,
) -> anyhow::Result<Vec<Path>> {
    if let Some(drivers) = current_drivers {
        if drivers.is_empty() {
            anyhow::bail!(
                "At least one driver must be provided in the [ ... ] list, or no list at all to use default drivers"
            );
        }
        Ok(drivers.into_iter().collect())
    } else if !compilation_data.default_drivers.is_empty() {
        let mut drivers = Vec::new();
        for driver_str in compilation_data.default_drivers.iter() {
            let driver_ident: syn::Path = syn::parse_str(driver_str).with_context(||{
                format!("easy_sql.ron is corrupted: Invalid driver name `{}`, expected valid Rust identifier",driver_str)
            })?;
            drivers.push(driver_ident);
        }

        Ok(drivers)
    } else {
        anyhow::bail!(
            "No default drivers provided in the build script, please provide supported drivers by using [ExampleDriver1,ExampleDriver2] at the start of the macro"
        );
    }
}

#[always_context]
pub fn table_join(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let input = parse_macro_input!(item as Input);

    let sql_crate = sql_crate();

    let item_name = input.struct_name;
    let main_table_struct = input.main_table;
    //input.joins

    let has_table_impls = input
        .joins
        .iter()
        .map(|join| {
            let table = match join {
                Join::Inner { table, .. } => table,
                Join::Left { table, .. } => table,
                Join::Right { table, .. } => table,
                Join::Cross { table } => table,
            };
            quote! {
                #table
            }
        })
        .collect::<Vec<_>>();

    let mut checks = Vec::new();
    let mut binds = Vec::new();

    let mut not_optional_joined_tables = vec![&main_table_struct];
    let mut optional_joined_tables = Vec::new();

    for join in input.joins.iter() {
        match join {
            Join::Inner { table, on: _ } => {
                not_optional_joined_tables.push(table);
            }
            Join::Left { table, on: _ } => {
                optional_joined_tables.push(table);
            }
            Join::Right { table, on: _ } => {
                //The right joined table takes priority
                optional_joined_tables.append(&mut not_optional_joined_tables);
                not_optional_joined_tables.push(table);
            }
            Join::Cross { table } => {
                not_optional_joined_tables.push(table);
            }
        }
    }

    let has_table_joined_impls = not_optional_joined_tables
        .iter()
        .map(|table| {
            quote! {
                impl #sql_crate::HasTableJoined<#table> for #item_name{
                    type MaybeOption<Y> = Y;

                    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>{
                        t
                    }
                }
            }
        })
        .chain(optional_joined_tables.iter().map(|table| {
            quote! {
                impl #sql_crate::HasTableJoined<#table> for #item_name{
                    type MaybeOption<Y> = Option<Y>;

                    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>{
                        Some(t)
                    }
                }
            }
        }))
        .collect::<Vec<_>>();

    let mut result = TokensBuilder::default();

    let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;

    let supported_drivers = supported_drivers(input.drivers.clone(), &compilation_data)?
        .into_iter()
        .map(|e| e.into_token_stream())
        .collect::<Vec<_>>();

    result.add(quote! {
        struct #item_name;

        #(#has_table_joined_impls)*
    });

    let mut current_param_n = 0;
    let mut current_format_params = Vec::new();
    let mut before_param_n = quote! {};
    let mut before_format = Vec::new();

    let driver = ProvidedDrivers::SingleWithChecks {
        driver: quote! { D },
        checks: supported_drivers,
    };

    let table_joins = input
        .joins
        .iter()
        .map(|join| {
            let format_str = match join {
                Join::Inner { table, on } => {
                    current_format_params.push(driver.table_name(&sql_crate, table));
                    let on = on.clone().into_query_string(
                        &mut binds,
                        &mut checks,
                        &sql_crate,
                        &driver,
                        &mut current_param_n,
                        &mut current_format_params,
                        &mut before_param_n,
                        &mut before_format,
                        false,
                        false,
                        None,
                        None,
                    );

                    format!(" INNER JOIN {{}} ON {}", on)
                }
                Join::Left { table, on } => {
                    current_format_params.push(driver.table_name(&sql_crate, table));
                    let on = on.clone().into_query_string(
                        &mut binds,
                        &mut checks,
                        &sql_crate,
                        &driver,
                        &mut current_param_n,
                        &mut current_format_params,
                        &mut before_param_n,
                        &mut before_format,
                        false,
                        false,
                        None,
                        None,
                    );

                    format!(" LEFT JOIN {{}} ON {}", on)
                }
                Join::Right { table, on } => {
                    current_format_params.push(driver.table_name(&sql_crate, table));
                    let on = on.clone().into_query_string(
                        &mut binds,
                        &mut checks,
                        &sql_crate,
                        &driver,
                        &mut current_param_n,
                        &mut current_format_params,
                        &mut before_param_n,
                        &mut before_format,
                        false,
                        false,
                        None,
                        None,
                    );

                    format!(" RIGHT JOIN {{}} ON {}", on)
                }
                Join::Cross { table } => {
                    current_format_params.push(driver.table_name(&sql_crate, table));
                    format!(" CROSS JOIN {{}}")
                }
            };
            format_str
        })
        .collect::<Vec<_>>();

    let table_joins_str = table_joins.join("");

    result.add(quote! {
        impl<D: #sql_crate::Driver> #sql_crate::Table<D> for #item_name {
            fn table_name() -> &'static str {
                <#main_table_struct as #sql_crate::Table<D>>::table_name()
            }

            fn primary_keys() -> Vec<&'static str>{
                vec![]
            }

            #[inline(always)]
            fn table_joins(current_query: &mut String ) {
                let _ = |___t___:#item_name|{
                    #(#checks)*
                };

                #(#binds)*

                #(#before_format)*

                // Define the identifier delimiter for the current driver
                let _easy_sql_d = <D as #sql_crate::Driver>::identifier_delimiter();

                let result = format!(#table_joins_str, #(#current_format_params),*);

                current_query.push_str(&result);
            }
        }

        impl #sql_crate::HasTable<#main_table_struct> for #item_name{}

        #(impl #sql_crate::HasTable<#has_table_impls> for #item_name{})*


    });

    // panic!("{}", result.to_string());

    Ok(result.finalize().into())
}
