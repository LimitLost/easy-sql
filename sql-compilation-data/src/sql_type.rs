#[cfg(feature = "data")]
use easy_macros::{anyhow, macros::always_context};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
            SqlType::I8 => "INT",
            SqlType::I16 => "INT",
            SqlType::I32 => "INT",
            SqlType::I64 => "INT",
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
