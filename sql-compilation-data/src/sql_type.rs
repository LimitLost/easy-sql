#[cfg(feature = "data")]
use ::{
    anyhow::{self, Context},
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    syn,
};
#[cfg(feature = "data")]
use easy_macros::{helpers::context, macros::always_context};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SqlRangeType {
    ///int4range
    I32,
    ///int8range
    I64,
    ///daterange
    NaiveDate,
    ///tsrange
    NaiveDateTime,
    ///numrange
    BigDecimal,
    ///numrange
    Decimal,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SqlType {
    ///Postgresql: inet
    ///Sqlite: BLOB
    IpAddr,
    ///Postgresql: boolean
    ///Sqlite: BOOLEAN
    Bool,
    ///Postgresql: float4
    ///Sqlite: FLOAT
    F32,
    ///Postgresql: float8
    ///Sqlite: DOUBLE
    F64,
    ///Postgresql: char
    ///Sqlite: INT
    I8,
    ///Postgresql: smallint
    ///Sqlite: INT
    I16,
    ///Postgresql: integer
    ///Sqlite: INT
    I32,
    ///Postgresql: bigint
    ///Sqlite: INT
    I64,
    ///Postgresql: text
    ///Sqlite: TEXT
    String,
    ///Aka Duration or TimeDelta
    ///Postgresql: interval
    ///Sqlite: BLOB
    Interval,
    /// Vec<u8>
    ///Postgresql: bytea
    ///Sqlite: BLOB
    Bytes,
    ///Postgresql: type[]
    ///Sqlite: BLOB
    List(Box<SqlType>),
    ///Postgresql: type[x]
    ///Sqlite: BLOB
    Array {
        data_type: Box<SqlType>,
        size: usize,
    },
    ///Postgresql: date
    ///Sqlite: BLOB
    NaiveDate,
    ///Postgresql: timestamp
    ///Sqlite: BLOB
    NaiveDateTime,
    ///Postgresql: time
    ///Sqlite: BLOB
    NaiveTime,
    ///Postgresql: uuid
    ///Sqlite: BLOB
    Uuid,
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    Decimal,
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    BigDecimal,
    ///Postgresql: <See TableFieldRangeType>
    ///Sqlite: BLOB
    Range(SqlRangeType),
    //
    //Not Implemented:
    //
    //PgCube
    //IpNetwork
    //Oid
    //PgCiText
    //PgHstore
    //PgInterval
    //PgLQuery
    //PgLTree
    //PgLine
    //PgMoney
    //PgPoint
    //PgRange<Date>
    //PgRange<OffsetDateTime>
    //PgRange<PrimitiveDateTime>
    //PgTimeTz
    //PgTimeTz<NaiveTime, FixedOffset>
    //MacAddress
    //BitVec
    //Date
    //OffsetDateTime
    //PrimitiveDateTime
    //Time
}

impl SqlType {
    pub fn sqlite(self) -> &'static str {
        match self {
            SqlType::IpAddr => "BLOB",
            SqlType::Bool => "BOOLEAN",
            SqlType::F32 => "FLOAT",
            SqlType::F64 => "DOUBLE",
            SqlType::I8 => "INTEGER",
            SqlType::I16 => "INTEGER",
            SqlType::I32 => "INTEGER",
            SqlType::I64 => "INTEGER",
            SqlType::String => "TEXT",
            SqlType::Interval => "BLOB",
            SqlType::Bytes => "BLOB",
            SqlType::List(_) => "BLOB",
            SqlType::Array { .. } => "BLOB",
            SqlType::NaiveDate => "BLOB",
            SqlType::NaiveDateTime => "BLOB",
            SqlType::NaiveTime => "BLOB",
            SqlType::Uuid => "BLOB",
            SqlType::Decimal => "BLOB",
            SqlType::BigDecimal => "BLOB",
            SqlType::Range(_) => "BLOB",
        }
    }

    pub fn postgres(&self, is_auto_increment: bool) -> String {
        match self {
            SqlType::IpAddr => "inet".to_string(),
            SqlType::Bool => "boolean".to_string(),
            SqlType::F32 => "real".to_string(),
            SqlType::F64 => "double precision".to_string(),
            SqlType::I8 => "char".to_string(),
            SqlType::I16 => "smallint".to_string(),
            SqlType::I32 => if is_auto_increment { "SERIAL".to_string() } else { "integer".to_string() },
            SqlType::I64 => if is_auto_increment { "BIGSERIAL".to_string() } else { "bigint".to_string() },
            SqlType::String => "text".to_string(),
            SqlType::Interval => "interval".to_string(),
            SqlType::Bytes => "bytea".to_string(),
            SqlType::List(inner) => format!("{}[]", inner.postgres(false)),
            SqlType::Array { data_type, size } => format!("{}[{}]", data_type.postgres(false), size),
            SqlType::NaiveDate => "date".to_string(),
            SqlType::NaiveDateTime => "timestamp".to_string(),
            SqlType::NaiveTime => "time".to_string(),
            SqlType::Uuid => "uuid".to_string(),
            SqlType::Decimal => "numeric".to_string(),
            SqlType::BigDecimal => "numeric".to_string(),
            SqlType::Range(range_type) => match range_type {
                SqlRangeType::I32 => "int4range".to_string(),
                SqlRangeType::I64 => "int8range".to_string(),
                SqlRangeType::NaiveDate => "daterange".to_string(),
                SqlRangeType::NaiveDateTime => "tsrange".to_string(),
                SqlRangeType::BigDecimal => "numrange".to_string(),
                SqlRangeType::Decimal => "numrange".to_string(),
            },
        }
    }

    #[cfg(feature = "data")]
    #[always_context]
    pub fn from_syn_type(ty: &syn::Type) -> anyhow::Result<(Option<SqlType>, bool)> {
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
                                        last_segment =
                                            type_path.path.segments.last().with_context(
                                                context!(
                                                    "Type path is empty | ty: {:?}",
                                                    type_path
                                                ),
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

                Ok((ty_str_enum_value(&name_str, &generic_arg)?, not_null))
            }
            _ => {
                anyhow::bail!("Unsupported type: {}", ty.to_token_stream())
            }
        }
    }

    #[cfg(feature = "data")]
    ///`sql_type_parent` - Example: `quote::quote! {easy_lib::sql}`
    pub fn to_tokens(&self, sql_type_parent: &TokenStream) -> TokenStream {
        match self {
            SqlType::IpAddr => quote! {
                #sql_type_parent::SqlType::IpAddr
            },
            SqlType::Bool => quote! {
                #sql_type_parent::SqlType::Bool
            },
            SqlType::F32 => quote! {
                #sql_type_parent::SqlType::F32
            },
            SqlType::F64 => quote! {
                #sql_type_parent::SqlType::F64
            },
            SqlType::I8 => quote! {
                #sql_type_parent::SqlType::I8
            },
            SqlType::I16 => quote! {
                #sql_type_parent::SqlType::I16
            },
            SqlType::I32 => quote! {
                #sql_type_parent::SqlType::I32
            },
            SqlType::I64 => quote! {
                #sql_type_parent::SqlType::I64
            },
            SqlType::String => quote! {
                #sql_type_parent::SqlType::String
            },
            SqlType::Interval => quote! {
                #sql_type_parent::SqlType::Interval
            },
            SqlType::Bytes => quote! {
                #sql_type_parent::SqlType::Bytes
            },
            SqlType::List(sql_type) => {
                let sql_type = sql_type.to_tokens(sql_type_parent);
                quote! {
                    #sql_type_parent::SqlType::List(#sql_type)
                }
            }
            SqlType::Array { data_type, size } => {
                let data_type = data_type.to_tokens(sql_type_parent);
                quote! {
                    #sql_type_parent::SqlType::Array {
                        data_type: #data_type,
                        size: #size,
                    }
                }
            }
            SqlType::NaiveDate => {
                quote! {
                    #sql_type_parent::SqlType::NaiveDate
                }
            }
            SqlType::NaiveDateTime => {
                quote! {
                    #sql_type_parent::SqlType::NaiveDateTime
                }
            }
            SqlType::NaiveTime => {
                quote! {
                    #sql_type_parent::SqlType::NaiveTime
                }
            }
            SqlType::Uuid => {
                quote! {
                    #sql_type_parent::SqlType::Uuid
                }
            }
            SqlType::Decimal => {
                quote! {
                    #sql_type_parent::SqlType::Decimal
                }
            }
            SqlType::BigDecimal => {
                quote! {
                    #sql_type_parent::SqlType::BigDecimal
                }
            }
            SqlType::Range(sql_range_type) => {
                let sql_range_type = match sql_range_type {
                    SqlRangeType::I32 => quote! { #sql_type_parent::SqlRangeType::I32 },
                    SqlRangeType::I64 => quote! { #sql_type_parent::SqlRangeType::I64 },
                    SqlRangeType::NaiveDate => quote! { #sql_type_parent::SqlRangeType::NaiveDate },
                    SqlRangeType::NaiveDateTime => {
                        quote! { #sql_type_parent::SqlRangeType::NaiveDateTime }
                    }
                    SqlRangeType::BigDecimal => {
                        quote! { #sql_type_parent::SqlRangeType::BigDecimal }
                    }
                    SqlRangeType::Decimal => {
                        quote! { #sql_type_parent::SqlRangeType::Decimal }
                    }
                };
                quote! {
                    #sql_type_parent::SqlType::Range(#sql_range_type)
                }
            }
        }
    }
}
#[cfg(feature = "data")]
#[always_context]
fn ty_str_enum_value(
    ty_str: &str,
    generic_arg: &Option<String>,
) -> anyhow::Result<Option<SqlType>> {
    Ok(match ty_str {
        "IpAddr" => Some(SqlType::IpAddr),
        "bool" => Some(SqlType::Bool),
        "f32" => Some(SqlType::F32),
        "f64" => Some(SqlType::F64),
        "i8" => Some(SqlType::I8),
        "i16" => Some(SqlType::I16),
        "i32" => Some(SqlType::I32),
        "i64" => Some(SqlType::I64),
        "String" => Some(SqlType::String),
        "Interval" => Some(SqlType::Interval),
        "Vec<u8>" => Some(SqlType::Bytes),
        "NaiveDate" => Some(SqlType::NaiveDate),
        "NaiveDateTime" => Some(SqlType::NaiveDateTime),
        "NaiveTime" => Some(SqlType::NaiveTime),
        "Uuid" => Some(SqlType::Uuid),
        "Decimal" => Some(SqlType::Decimal),
        "BigDecimal" => Some(SqlType::BigDecimal),
        "Vec" => {
            if let Some(arg) = generic_arg {
                let subtype = ty_str_enum_value(&arg, &None::<String>)?;
                if let Some(subtype) = subtype {
                    Some(SqlType::List(Box::new(subtype)))
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
                    "i32" => Some(SqlType::Range(SqlRangeType::I32)),
                    "i64" => Some(SqlType::Range(SqlRangeType::I64)),
                    "NaiveDate" => Some(SqlType::Range(SqlRangeType::NaiveDate)),
                    "NaiveDateTime" => Some(SqlType::Range(SqlRangeType::NaiveDateTime)),
                    "BigDecimal" => Some(SqlType::Range(SqlRangeType::BigDecimal)),
                    "Decimal" => Some(SqlType::Range(SqlRangeType::Decimal)),

                    _ => None,
                }
            } else {
                anyhow::bail!("No Generic argument or Invalid/Not supported argument for Vec type")
            }
        }
        _ => None,
    })
}
