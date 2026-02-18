use anyhow::Context;
use easy_macros::{always_context, get_attributes};
use easy_sql_compilation_data::CompilationData;
use syn::{ItemStruct, Path, punctuated::Punctuated};

#[always_context]
pub fn supported_drivers(
    item: &ItemStruct,
    compilation_data: &CompilationData,
    optional: bool,
) -> anyhow::Result<Vec<Path>> {
    if let Some(attr_data) = get_attributes!(item, #[sql(drivers = __unknown__)])
        .into_iter()
        .next()
    {
        struct DriversParsed {
            drivers: Punctuated<Path, syn::Token![,]>,
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
    } else if !optional {
        anyhow::bail!(
            "No default drivers provided in the build script, please provide supported drivers using #[sql(drivers = ExampleDriver1,ExampleDriver2])] attribute"
        );
    } else {
        Ok(vec![])
    }
}
