use crate::{
    macros_components::{
        ProvidedDrivers, QueryType, generate_delete, generate_exists, generate_insert,
        generate_select, generate_update,
    },
    sql_crate,
};

use anyhow::Context;
use easy_macros::always_context;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use sql_compilation_data::CompilationData;
use syn::{self, parse::Parse};

/// Input structure for query! macro: optional driver, connection, query_type
struct QueryInput {
    driver: Option<syn::Path>,
    connection: syn::Expr,
    query: QueryType,
}

#[always_context]
impl Parse for QueryInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Check for optional driver specification: <Driver>
        let driver = if input.peek(syn::Token![<]) {
            input.parse::<syn::Token![<]>()?;
            let driver = input.parse::<syn::Path>()?;
            input.parse::<syn::Token![>]>()?;
            Some(driver)
        } else {
            None
        };

        let connection = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let query = input.parse::<QueryType>()?;
        Ok(QueryInput {
            driver,
            connection,
            query,
        })
    }
}

#[always_context]
pub fn query(input_raw: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let input_str = input_raw.to_string();
    let input = easy_macros::parse_macro_input!(input_raw as QueryInput);

    let connection = input.connection.into_token_stream();

    // Load compilation data to get driver information
    let sql_crate = sql_crate();

    // Use provided driver or load from compilation data
    let driver = if let Some(driver_path) = input.driver {
        ProvidedDrivers::Single(quote! {#driver_path})
    } else {
        let compilation_data = CompilationData::load(Vec::<String>::new(), false).with_context(|| {
            "Failed to load compilation data for query! macro. Make sure easy_sql_build::build is called in the build script"
        })?;

        match compilation_data.default_drivers.len() {
            1 => {
                let driver = compilation_data
                    .default_drivers
                    .first()
                    .context("No default driver found despite length being 1 (Unreachable)")?;
                let driver_parsed: TokenStream = syn::parse_str(driver)
                    .with_context(|| format!("Failed to parse driver path: {}", driver))?;
                ProvidedDrivers::MultipleWithConn {
                    drivers: vec![driver_parsed],
                    conn: connection.clone(),
                }
            }
            _ => {
                let mut parsed_drivers = Vec::new();

                for driver_str in compilation_data.default_drivers.iter() {
                    let driver_path: TokenStream = syn::parse_str(driver_str)
                        .with_context(|| format!("Failed to parse driver path: {}", driver_str))?;
                    parsed_drivers.push(driver_path);
                }

                ProvidedDrivers::MultipleWithConn {
                    drivers: parsed_drivers,
                    conn: connection.clone(),
                }
            }
        }
    };

    let result = match input.query {
        QueryType::Select(select) => generate_select(
            select.clone(),
            Some(&connection),
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Insert(insert) => generate_insert(
            insert.clone(),
            Some(&connection),
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Update(update) => generate_update(
            update.clone(),
            Some(&connection),
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Delete(delete) => generate_delete(
            delete.clone(),
            Some(&connection),
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Exists(exists) => generate_exists(
            exists.clone(),
            &connection,
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
    };

    Ok(result.into())
}
