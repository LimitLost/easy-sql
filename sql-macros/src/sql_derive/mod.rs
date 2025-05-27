mod database_setup;
mod sql_insert;
mod sql_output;
mod sql_table;
mod sql_update;

use convert_case::{Case, Casing};
pub use database_setup::*;
use easy_macros::{
    anyhow::{self, Context},
    helpers::context,
    macros::always_context,
    proc_macro2::TokenStream,
    quote::{self, ToTokens, quote},
    syn,
};
pub use sql_insert::*;
pub use sql_output::*;
pub use sql_table::*;
pub use sql_update::*;

enum TyData {
    Binary,
    IntoNoRef,
    IntoRef,
}

#[always_context]
impl TyData {
    fn bytes(&self) -> bool {
        match self {
            TyData::Binary => true,
            TyData::IntoNoRef => false,
            TyData::IntoRef => false,
        }
    }
}

#[always_context]
fn ty_name_into_data(
    ty: &str,
    generic_arg: &Option<String>,
    bytes_allowed: bool,
) -> anyhow::Result<TyData> {
    match ty {
        "Option" => match generic_arg {
            Some(arg) => {
                //Ignore option, use the sub type
                ty_name_into_data(arg, &None::<String>, bytes_allowed)
            }
            None => {
                anyhow::bail!("No Generic argument or Invalid argument for Option type")
            }
        },
        //Handle both bytes and accepted type list
        "Vec" => match generic_arg {
            Some(arg) => {
                let subtype_bytes = ty_name_into_data(arg, &None::<String>, true)?.bytes();
                if bytes_allowed {
                    if subtype_bytes {
                        Ok(TyData::Binary)
                    } else {
                        Ok(TyData::IntoRef)
                    }
                } else if subtype_bytes {
                    anyhow::bail!(
                        "Vec Generic Argument `{}` is not supported, use #[sql(bytes)] to convert Vector into bytes",
                        arg
                    );
                } else {
                    Ok(TyData::IntoRef)
                }
            }
            None => {
                anyhow::bail!("No Generic argument or Invalid argument for Vec type")
            }
        },
        "IpAddr" | "bool" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "String"
        | "PgInterval" | "Array" | "NaiveDate" | "NaiveDateTime" | "NaiveTime" | "Uuid"
        | "Decimal" | "BigDecimal" => Ok(TyData::IntoRef),
        "PgRange" => match generic_arg {
            Some(arg) => match arg.as_str() {
                "i32" | "i64" | "NaiveDate" | "NaiveDateTime" | "BigDecimal" | "Decimal" => {
                    Ok(TyData::IntoRef)
                }
                _ => {
                    if bytes_allowed {
                        Ok(TyData::Binary)
                    } else {
                        anyhow::bail!(
                            "PgRange Generic Argument `{}` is not supported, use #[sql(bytes)] to convert range into bytes",
                            arg
                        );
                    }
                }
            },
            None => {
                anyhow::bail!("No Generic argument or Invalid argument for PgRange type")
            }
        },
        "Range" => match generic_arg {
            Some(arg) => match arg.as_str() {
                "i32" | "i64" | "NaiveDate" | "NaiveDateTime" | "BigDecimal" | "Decimal" => {
                    Ok(TyData::IntoNoRef)
                }
                _ => {
                    if bytes_allowed {
                        Ok(TyData::Binary)
                    } else {
                        anyhow::bail!(
                            "Range Generic Argument `{}` is not supported, use #[sql(bytes)] to convert range into bytes",
                            arg
                        );
                    }
                }
            },
            None => {
                anyhow::bail!("No Generic argument or Invalid argument for Range type")
            }
        },
        unknown_ty => {
            if bytes_allowed {
                Ok(TyData::Binary)
            } else {
                anyhow::bail!(
                    "Unknown type {} is not supported, use #[sql(bytes)] to convert it into bytes",
                    unknown_ty
                )
            }
        }
    }
}

#[always_context]
fn ty_to_variant(
    field_name: TokenStream,
    ty: &syn::Type,
    crate_prefix: &TokenStream,
    bytes_allowed: bool,
) -> anyhow::Result<TokenStream> {
    match ty {
        syn::Type::Array(_) => {
            //Convert into Vec
            anyhow::bail!("Arrays are not supported yet")
        }
        syn::Type::Paren(type_paren) => {
            ty_to_variant(field_name, &type_paren.elem, crate_prefix, bytes_allowed)
        }
        syn::Type::Path(type_path) => {
            let name = type_path
                .path
                .segments
                .last()
                .with_context(context!("Type path is empty | ty: {:?}", type_path))?;

            //Get the last segment of the path in generic arg
            let generic_arg = match &name.arguments {
                syn::PathArguments::None => None,
                syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                    angle_bracketed_generic_arguments
                        .args
                        .first()
                        .map(|el| match el {
                            syn::GenericArgument::Type(ty) => match ty {
                                syn::Type::Path(type_path) => type_path
                                    .path
                                    .segments
                                    .last()
                                    .map(|name| name.ident.to_string()),
                                _ => None,
                            },
                            _ => None,
                        })
                        .flatten()
                }
                syn::PathArguments::Parenthesized(_) => None,
            };

            let found = ty_name_into_data(&name.ident.to_string(), &generic_arg, bytes_allowed)?;

            Ok(match found {
                TyData::Binary => {
                    quote! {
                        easy_lib::sql::SqlValueMaybeRef::Value(easy_lib::sql::SqlValue::Bytes(easy_lib::sql::to_binary(&self.#field_name)?))
                    }
                }
                TyData::IntoNoRef => {
                    quote! {
                        self.#field_name.into()
                    }
                }
                TyData::IntoRef => {
                    quote! {
                        (&self.#field_name).into()
                    }
                }
            })
        }
        syn::Type::Reference(_) => {
            //(&) into ref
            anyhow::bail!("References are not supported yet")
        }
        t => {
            anyhow::bail!("Unsupported type: {}", t.to_token_stream())
        }
    }
}

#[always_context]
fn ty_str_enum_value(
    ty_str: &str,
    generic_arg: &Option<String>,
    sql_crate: &TokenStream,
) -> anyhow::Result<Option<TokenStream>> {
    Ok(match ty_str {
        "IpAddr" | "bool" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "String"
        | "Interval" | "NaiveDate" | "NaiveDateTime" | "NaiveTime" | "Uuid" | "Decimal"
        | "BigDecimal" => {
            let name_variant = quote::format_ident!("{}", ty_str.to_case(Case::Pascal));

            Some(quote! {#name_variant})
        }
        "Vec" => {
            if let Some(arg) = generic_arg {
                let subtype = ty_str_enum_value(&arg, &None::<String>, sql_crate)?;
                if let Some(subtype) = subtype {
                    Some(quote! { List(Box::new(#subtype)) })
                } else {
                    None
                }
            } else {
                anyhow::bail!("No Generic argument or Invalid/Not supported argument for Vec type")
            }
        }
        "Range" | "PgRange" => {
            if let Some(arg) = generic_arg {
                match arg.as_str() {
                    "i32" | "i64" | "NaiveDate" | "NaiveDateTime" | "BigDecimal" | "Decimal" => {
                        let name_variant = quote::format_ident!("{}", ty_str.to_case(Case::Pascal));
                        Some(quote! {Range(#sql_crate::SqlRangeType::#name_variant)})
                    }
                    _ => None,
                }
            } else {
                anyhow::bail!("No Generic argument or Invalid/Not supported argument for Vec type")
            }
        }
        _ => None,
    })
}

#[always_context]
///Returns only type enum variant
fn ty_enum_value(
    ty: &syn::Type,
    sql_crate: &TokenStream,
) -> anyhow::Result<(Option<TokenStream>, bool)> {
    match ty {
        syn::Type::Path(type_path) => {
            let mut last_segment = type_path
                .path
                .segments
                .last()
                .with_context(context!("Type path is empty | ty: {:?}", type_path))?;

            let mut name_str = last_segment.ident.to_string();

            let mut not_null = true;

            if name_str == "Option" {
                match &last_segment.arguments {
                    syn::PathArguments::None => {
                        anyhow::bail!("Option with no generic argument is not supported")
                    }
                    syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                        match angle_bracketed_generic_arguments.args.first()? {
                            syn::GenericArgument::Type(ty) => match ty {
                                syn::Type::Path(type_path) => {
                                    last_segment = type_path.path.segments.last().with_context(
                                        context!("Type path is empty | ty: {:?}", type_path),
                                    )?;

                                    name_str = last_segment.ident.to_string();

                                    not_null = false;
                                }
                                _ => anyhow::bail!(
                                    "Option with non- type path generic argument is not supported"
                                ),
                            },
                            _ => anyhow::bail!(
                                "Option with non-type generic argument is not supported"
                            ),
                        }
                    }
                    syn::PathArguments::Parenthesized(_) => {
                        anyhow::bail!(
                            "Option with parenthesized generic arguments is not supported"
                        )
                    }
                }
            }

            let generic_arg = match &last_segment.arguments {
                syn::PathArguments::None => None,
                syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                    angle_bracketed_generic_arguments
                        .args
                        .first()
                        .map(|el| match el {
                            syn::GenericArgument::Type(ty) => match ty {
                                syn::Type::Path(type_path) => type_path
                                    .path
                                    .segments
                                    .last()
                                    .map(|name| name.ident.to_string()),
                                _ => None,
                            },
                            _ => None,
                        })
                        .flatten()
                }
                syn::PathArguments::Parenthesized(_) => None,
            };

            Ok((
                ty_str_enum_value(&name_str, &generic_arg, sql_crate)?,
                not_null,
            ))
        }
        _ => {
            anyhow::bail!("Unsupported type: {}", ty.to_token_stream())
        }
    }
}
