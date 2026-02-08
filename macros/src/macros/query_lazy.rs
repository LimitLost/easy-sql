use crate::{
    macros_components::{
        ProvidedDrivers, QueryType, generate_delete, generate_insert, generate_select,
        generate_update,
    },
    sql_crate,
};

use anyhow::Context;
use easy_macros::always_context;
use easy_sql_compilation_data::CompilationData;
use quote::quote;
use syn::{self, parse::Parse};

struct Input {
    driver: Option<syn::Path>,
    query: QueryType,
}

#[always_context]
impl Parse for Input {
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

        let query = input.parse::<QueryType>()?;
        Ok(Input { driver, query })
    }
}

#[always_context]
pub fn query_lazy(input_raw: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let input_str = input_raw.to_string();
    let input = easy_macros::parse_macro_input!(input_raw as Input);

    // Load compilation data to get driver information
    let sql_crate = sql_crate();

    // Use provided driver or load from compilation data
    let driver = if let Some(driver_path) = input.driver {
        ProvidedDrivers::Single(quote! {#driver_path})
    } else {
        let compilation_data = CompilationData::load(Vec::<String>::new(), false).with_context(|| {
            "Failed to load compilation data for query_lazy! macro. Make sure easy_sql_build::build is called in the build script"
        })?;

        if compilation_data.default_drivers.len() > 1 {
            return Err(anyhow::anyhow!(
                "Multiple default drivers found in compilation data. Please specify the driver at the macro call site using <Driver> syntax (before connection) or limit to a single default driver in the build script"
            ));
        }

        if let Some(driver_str) = compilation_data.default_drivers.first() {
            let driver_path: syn::Path = syn::parse_str(driver_str)
                .with_context(|| format!("Failed to parse driver path: {}", driver_str))?;
            ProvidedDrivers::Single(quote! {#driver_path})
        } else {
            return Err(anyhow::anyhow!(
                "No default driver found in compilation data. Please specify a driver in the build script or at the macro call site using <Driver> syntax (before connection)"
            ));
        }
    };

    let result = match input.query {
        QueryType::Select(select) => generate_select(
            select.clone(),
            #[context(no)]
            None,
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Insert(insert) => generate_insert(
            insert.clone(),
            #[context(no)]
            None,
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Update(update) => generate_update(
            update.clone(),
            #[context(no)]
            None,
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Delete(delete) => generate_delete(
            delete.clone(),
            #[context(no)]
            None,
            driver.clone(),
            &sql_crate,
            &input_str,
        )?,
        QueryType::Exists(_) => {
            anyhow::bail!(
                "exists queries are not supported in query_lazy! macro, use query! macro instead"
            )
        }
    };

    Ok(result.into())
}
