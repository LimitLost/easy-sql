mod database_setup;
mod sql_insert;
mod sql_output;
mod sql_table;
mod sql_update;

use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::quote,
    syn::{self},
};
pub use database_setup::*;
use easy_macros::macros::{always_context, get_attributes};
use sql_compilation_data::CompilationData;
pub use sql_insert::*;
pub use sql_output::*;
pub use sql_table::*;
pub use sql_update::*;
use syn::{ItemStruct, Path, punctuated::Punctuated};

#[always_context]
fn ty_to_variant(
    field_name: TokenStream,
    bytes: bool,
    crate_prefix: &TokenStream,
) -> anyhow::Result<TokenStream> {
    if bytes {
        Ok(quote! {
            #crate_prefix::to_binary(&self.#field_name)?
        })
    } else {
        Ok(quote! {
            self.#field_name
        })
    }
}

#[always_context]
fn supported_drivers(
    item: &ItemStruct,
    compilation_data: &CompilationData,
) -> anyhow::Result<Vec<Path>> {
    if let Some(attr_data) = get_attributes!(item, #[sql(drivers = __unknown__)])
        .into_iter()
        .next()
    {
        struct DriversParsed {
            drivers: Punctuated<syn::Path, syn::Token![,]>,
        }

        impl syn::parse::Parse for DriversParsed {
            fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                let drivers = Punctuated::parse_terminated(input)?;
                Ok(DriversParsed { drivers })
            }
        }

        let DriversParsed { drivers } = syn::parse2(attr_data.clone())
            .context("Invalid drivers provided, expected comma separated list of identifiers")?;
        if drivers.is_empty() {
            anyhow::bail!(
                "At least one driver must be provided in the #[sql(drivers = ...)] attribute"
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
            "No default drivers provided in the build script, please provide supported drivers using #[sql(drivers = ExampleDriver1,ExampleDriver2])] attribute"
        );
    }
}
