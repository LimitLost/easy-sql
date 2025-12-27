use crate::{
    query_macro_components::{
        QueryType, generate_delete, generate_exists, generate_insert, generate_select,
        generate_update,
    },
    sql_crate,
};

use anyhow::Context;
use easy_macros::always_context;
use quote::quote;
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

    let connection = input.connection;

    // Load compilation data to get driver information
    let sql_crate = sql_crate();

    // Use provided driver or load from compilation data
    let driver = if let Some(driver_path) = input.driver {
        quote! {#driver_path}
    } else {
        let compilation_data = CompilationData::load(Vec::<String>::new(), false).with_context(|| {
            "Failed to load compilation data for query! macro. Make sure easy_sql::build is called in build.rs"
        })?;

        if let Some(driver_str) = compilation_data.default_drivers.first() {
            let driver_path: syn::Path = syn::parse_str(driver_str)
                .with_context(|| format!("Failed to parse driver path: {}", driver_str))?;
            quote! {#driver_path}
        } else {
            return Err(anyhow::anyhow!(
                "No default driver found in compilation data. Please specify a driver in build.rs, or at the macro call site using <Driver> syntax (before connection)"
            ));
        }
    };

    let result = match input.query {
        QueryType::Select(select) => generate_select(
            select.clone(),
            Some(&connection),
            &driver,
            &sql_crate,
            &input_str,
        )?,
        QueryType::Insert(insert) => generate_insert(
            insert.clone(),
            Some(&connection),
            &driver,
            &sql_crate,
            &input_str,
        )?,
        QueryType::Update(update) => generate_update(
            update.clone(),
            Some(&connection),
            &driver,
            &sql_crate,
            &input_str,
        )?,
        QueryType::Delete(delete) => generate_delete(
            delete.clone(),
            Some(&connection),
            &driver,
            &sql_crate,
            &input_str,
        )?,
        QueryType::Exists(exists) => {
            generate_exists(exists.clone(), &connection, &driver, &sql_crate, &input_str)?
        }
    };

    Ok(result.into())
}
