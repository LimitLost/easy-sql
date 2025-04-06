mod database_setup;
mod sql_insert;
mod sql_output;
mod sql_table;
mod sql_update;

use convert_case::Casing;
pub use database_setup::*;
use easy_macros::{
    anyhow::{self, Context},
    helpers::context,
    macros::always_context,
    proc_macro2::{self, TokenStream},
    quote::{self, quote},
    syn,
};
pub use sql_insert::*;
pub use sql_output::*;
pub use sql_table::*;
pub use sql_update::*;

enum IntoTy {
    NoRef,
    Ref,
}

struct TyData {
    into: Option<IntoTy>,
    bytes: bool,
}

#[always_context]
fn ty_name_into_data(
    ty: &str,
    generic_arg: Option<String>,
    bytes_allowed: bool,
) -> anyhow::Result<TyData> {
    match ty {
        //Handle both bytes and accepted type list
        "Vec" => match generic_arg {
            Some(arg) => {
                let subtype_bytes = ty_name_into_data(&arg, None::<String>, true)?.bytes;
                if bytes_allowed {
                    if subtype_bytes {
                        Ok(TyData {
                            into: None,
                            bytes: true,
                        })
                    } else {
                        Ok(TyData {
                            into: Some(IntoTy::Ref),
                            bytes: false,
                        })
                    }
                } else if subtype_bytes {
                    anyhow::bail!(
                        "Vec Generic Argument `{}` is not supported, use #[sql(bytes)] to convert Vector into bytes",
                        arg
                    );
                } else {
                    Ok(TyData {
                        into: Some(IntoTy::Ref),
                        bytes: false,
                    })
                }
            }
            None => {
                anyhow::bail!("No Generic argument or Invalid argument for Vec type")
            }
        },
        "IpAddr" | "bool" | "f32" | "f64" | "i8" | "i16" | "i32" | "i64" | "String"
        | "PgInterval" | "Array" | "NaiveDate" | "NaiveDateTime" | "NaiveTime" | "Uuid"
        | "Decimal" | "BigDecimal" => Ok(TyData {
            into: Some(IntoTy::Ref),
            bytes: false,
        }),
        "PgRange" => match generic_arg {
            Some(arg) => match arg.as_str() {
                "i32" | "i64" | "NaiveDate" | "NaiveDateTime" | "BigDecimal" | "Decimal" => {
                    Ok(TyData {
                        into: Some(IntoTy::Ref),
                        bytes: false,
                    })
                }
                _ => {
                    if bytes_allowed {
                        Ok(TyData {
                            into: None,
                            bytes: true,
                        })
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
                    Ok(TyData {
                        into: Some(IntoTy::NoRef),
                        bytes: false,
                    })
                }
                _ => {
                    if bytes_allowed {
                        Ok(TyData {
                            into: None,
                            bytes: true,
                        })
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
                Ok(TyData {
                    into: None,
                    bytes: true,
                })
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
        syn::Type::Array(type_array) => {
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
            let generic_arg = match name.arguments {
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
                syn::PathArguments::Parenthesized(parenthesized_generic_arguments) => None,
            };

            ty_name_into_data(&name.ident.to_string(), generic_arg, bytes_allowed)

            //TODO Handle type data, generate variant
        }
        syn::Type::Reference(type_reference) => todo!(),
        syn::Type::Slice(type_slice) => todo!(),
        syn::Type::TraitObject(type_trait_object) => todo!(),
        syn::Type::Tuple(type_tuple) => todo!(),
        syn::Type::Verbatim(token_stream) => todo!(),
        _ => todo!(),
    }
}

fn update_ty_to_variant(ty: &syn::Type) -> TokenStream {}
