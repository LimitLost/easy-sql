use std::ops::Bound;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, NaiveDateTime};
use easy_macros::macros::always_context;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{
    Encode,
    postgres::types::{PgInterval, PgRange},
};

use super::Db;
use crate::DriverValue;

#[derive(Serialize, Deserialize)]
#[serde(remote = "PgRange")]
struct PgRangeSerde<T> {
    start: Bound<T>,
    end: Bound<T>,
}
#[derive(Serialize, Deserialize)]
#[serde(remote = "PgInterval")]
struct PgIntervalSerde {
    months: i32,
    days: i32,
    microseconds: i64,
}

#[derive(Serialize, Deserialize)]
struct PgIntervalSerde2 {
    months: i32,
    days: i32,
    microseconds: i64,
}

#[always_context]
impl From<&PgInterval> for PgIntervalSerde2 {
    fn from(value: &PgInterval) -> Self {
        Self {
            months: value.months,
            days: value.days,
            microseconds: value.microseconds,
        }
    }
}

#[always_context]
impl From<PgInterval> for PgIntervalSerde2 {
    fn from(value: PgInterval) -> Self {
        Self {
            months: value.months,
            days: value.days,
            microseconds: value.microseconds,
        }
    }
}

fn binary<T: serde::Serialize>(v: T) -> Result<Vec<u8>, bincode::error::EncodeError> {
    bincode::serde::encode_to_vec(v, bincode::config::standard())
}
#[derive(Debug, Serialize, Deserialize)]
pub enum SqlRangeValue {
    ///int4range
    I32(#[serde(with = "PgRangeSerde")] PgRange<i32>),
    ///int8range
    I64(#[serde(with = "PgRangeSerde")] PgRange<i64>),
    ///daterange
    NaiveDate(#[serde(with = "PgRangeSerde")] PgRange<chrono::NaiveDate>),
    ///tsrange
    NaiveDateTime(#[serde(with = "PgRangeSerde")] PgRange<chrono::NaiveDateTime>),
    ///numrange
    BigDecimal(#[serde(with = "PgRangeSerde")] PgRange<sqlx::types::BigDecimal>),
    ///numrange
    Decimal(#[serde(with = "PgRangeSerde")] PgRange<sqlx::types::Decimal>),
}

#[derive(Debug, Serialize)]
pub enum SqlRangeValueRef<'a> {
    ///int4range
    I32(#[serde(with = "PgRangeSerde")] &'a PgRange<i32>),
    ///int8range
    I64(#[serde(with = "PgRangeSerde")] &'a PgRange<i64>),
    ///daterange
    NaiveDate(#[serde(with = "PgRangeSerde")] &'a PgRange<chrono::NaiveDate>),
    ///tsrange
    NaiveDateTime(#[serde(with = "PgRangeSerde")] &'a PgRange<chrono::NaiveDateTime>),
    ///numrange
    BigDecimal(#[serde(with = "PgRangeSerde")] &'a PgRange<sqlx::types::BigDecimal>),
    ///numrange
    Decimal(#[serde(with = "PgRangeSerde")] &'a PgRange<sqlx::types::Decimal>),
}
#[derive(Debug, Serialize)]
pub enum SqlValueRef<'a> {
    ///Postgresql: inet
    ///Sqlite: BLOB
    IpAddr(&'a std::net::IpAddr),
    ///Postgresql: boolean
    ///Sqlite: BOOLEAN
    Bool(&'a bool),
    ///Postgresql: float4
    ///Sqlite: REAL
    F32(&'a f32),
    ///Postgresql: float8
    ///Sqlite: REAL
    F64(&'a f64),
    ///Postgresql: char
    ///Sqlite: INTEGER
    I8(&'a i8),
    ///Postgresql: smallint
    ///Sqlite: INTEGER
    I16(&'a i16),
    ///Postgresql: integer
    ///Sqlite: INTEGER
    I32(&'a i32),
    ///Postgresql: bigint
    ///Sqlite: BIGINT
    I64(&'a i64),
    ///Postgresql: text
    ///Sqlite: TEXT
    String(&'a String),
    Str(&'a str),
    ///Aka Duration or TimeDelta
    ///Postgresql: interval
    ///Sqlite: BLOB
    Interval(#[serde(with = "PgIntervalSerde")] &'a PgInterval),
    /// Vec<u8>
    ///Postgresql: bytea
    ///Sqlite: BLOB
    Bytes(&'a Vec<u8>),
    ///Postgresql: type[]
    ///Sqlite: BLOB
    List(Vec<SqlValueRef<'a>>),
    ///Postgresql: type[x]
    ///Sqlite: BLOB
    Array(Vec<SqlValueRef<'a>>),
    ///Postgresql: date
    ///Sqlite: BLOB
    NaiveDate(&'a chrono::NaiveDate),
    ///Postgresql: timestamp
    ///Sqlite: BLOB
    NaiveDateTime(&'a chrono::NaiveDateTime),
    ///Postgresql: time
    ///Sqlite: BLOB
    NaiveTime(&'a chrono::NaiveTime),
    ///Postgresql: uuid
    ///Sqlite: BLOB
    Uuid(&'a uuid::Uuid),
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    Decimal(&'a sqlx::types::Decimal),
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    BigDecimal(&'a sqlx::types::BigDecimal),
    ///Postgresql: <See TableFieldRangeType>
    ///Sqlite: BLOB
    Range(SqlRangeValueRef<'a>),
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

#[derive(Debug, Serialize, Deserialize)]
pub enum SqlValue {
    ///Postgresql: inet
    ///Sqlite: BLOB
    IpAddr(std::net::IpAddr),
    ///Postgresql: boolean
    ///Sqlite: BOOLEAN
    Bool(bool),
    ///Postgresql: float4
    ///Sqlite: REAL
    F32(f32),
    ///Postgresql: float8
    ///Sqlite: REAL
    F64(f64),
    ///Postgresql: char
    ///Sqlite: INTEGER
    I8(i8),
    ///Postgresql: smallint
    ///Sqlite: INTEGER
    I16(i16),
    ///Postgresql: integer
    ///Sqlite: INTEGER
    I32(i32),
    ///Postgresql: bigint
    ///Sqlite: BIGINT
    I64(i64),
    ///Postgresql: text
    ///Sqlite: TEXT
    String(String),
    ///Aka Duration or TimeDelta
    ///Postgresql: interval
    ///Sqlite: BLOB
    Interval(#[serde(with = "PgIntervalSerde")] PgInterval),
    /// Vec<u8>
    ///Postgresql: bytea
    ///Sqlite: BLOB
    Bytes(Vec<u8>),
    ///Postgresql: type[]
    ///Sqlite: BLOB
    List(Vec<SqlValue>),
    ///Postgresql: type[x]
    ///Sqlite: BLOB
    Array(Vec<SqlValue>),
    ///Postgresql: date
    ///Sqlite: BLOB
    NaiveDate(chrono::NaiveDate),
    ///Postgresql: timestamp
    ///Sqlite: BLOB
    NaiveDateTime(chrono::NaiveDateTime),
    ///Postgresql: time
    ///Sqlite: BLOB
    NaiveTime(chrono::NaiveTime),
    ///Postgresql: uuid
    ///Sqlite: BLOB
    Uuid(uuid::Uuid),
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    Decimal(sqlx::types::Decimal),
    ///Postgresql: NUMERIC
    ///Sqlite: BLOB
    BigDecimal(sqlx::types::BigDecimal),
    ///Postgresql: <See TableFieldRangeType>
    ///Sqlite: BLOB
    Range(SqlRangeValue),
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

#[always_context]
impl<'a> Encode<'a, Db> for SqlValueRef<'a> {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlValueRef::IpAddr(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValueRef::Bool(v) => <bool as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::F32(v) => <f32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::F64(v) => <f64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I8(v) => <i8 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I16(v) => <i16 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I32(v) => <i32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I64(v) => <i64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::String(v) => <String as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Str(v) => <&str as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Interval(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(
                &binary(PgIntervalSerde2::from(*v))?,
                buf,
            ),
            SqlValueRef::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::List(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValueRef::Array(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValueRef::NaiveDate(v) => {
                <chrono::NaiveDate as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
            SqlValueRef::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
            SqlValueRef::NaiveTime(v) => {
                <chrono::NaiveTime as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
            SqlValueRef::Uuid(v) => <uuid::Uuid as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Decimal(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValueRef::BigDecimal(v) => {
                <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueRef::Range(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
        }
    }

    /* fn encode(
        self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        match self {
            SqlValue::IpAddr(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, Db>>::encode(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::String(v) => <String as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Interval(v) => {
                <Vec<u8> as Encode<'a, Db>>::encode(binary(PgIntervalSerde2::from(v))?, buf)
            }
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode(v, buf),
            SqlValue::List(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::Array(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, Db>>::encode(v, buf),
            SqlValue::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, Db>>::encode(v, buf)
            }
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Decimal(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::BigDecimal(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::Range(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
        }
    } */

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        Some(match self {
            SqlValueRef::IpAddr(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Bool(_) => <bool as sqlx::Type<Db>>::type_info(),
            SqlValueRef::F32(_) => <f32 as sqlx::Type<Db>>::type_info(),
            SqlValueRef::F64(_) => <f64 as sqlx::Type<Db>>::type_info(),
            SqlValueRef::I8(_) => <i8 as sqlx::Type<Db>>::type_info(),
            SqlValueRef::I16(_) => <i16 as sqlx::Type<Db>>::type_info(),
            SqlValueRef::I32(_) => <i32 as sqlx::Type<Db>>::type_info(),
            SqlValueRef::I64(_) => <i64 as sqlx::Type<Db>>::type_info(),
            SqlValueRef::String(_) => <String as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Str(_) => <&str as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Interval(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Bytes(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::List(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Array(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::NaiveDate(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValueRef::NaiveDateTime(_) => <chrono::NaiveDateTime as sqlx::Type<Db>>::type_info(),
            SqlValueRef::NaiveTime(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Uuid(_) => <uuid::Uuid as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Decimal(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::BigDecimal(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Range(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
        })
    }
}

#[always_context]
impl sqlx::Type<Db> for SqlValueRef<'_> {
    fn type_info() -> <Db as sqlx::Database>::TypeInfo {
        //Overriden by Encode anyway
        <Vec<u8> as sqlx::Type<Db>>::type_info()
    }
}

#[always_context]
impl<'a> Encode<'a, Db> for SqlValue {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlValue::IpAddr(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::String(v) => <String as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Interval(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(
                &binary(PgIntervalSerde2::from(*v))?,
                buf,
            ),
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::List(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValue::Array(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Decimal(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValue::BigDecimal(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValue::Range(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
        }
    }

    fn encode(
        self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        match self {
            SqlValue::IpAddr(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, Db>>::encode(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::String(v) => <String as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Interval(v) => {
                <Vec<u8> as Encode<'a, Db>>::encode(binary(PgIntervalSerde2::from(v))?, buf)
            }
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode(v, buf),
            SqlValue::List(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::Array(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, Db>>::encode(v, buf),
            SqlValue::NaiveDateTime(v) => <chrono::NaiveDateTime as Encode<'a, Db>>::encode(v, buf),
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Decimal(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::BigDecimal(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValue::Range(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
        }
    }

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        Some(match self {
            SqlValue::IpAddr(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::Bool(_) => <bool as sqlx::Type<Db>>::type_info(),
            SqlValue::F32(_) => <f32 as sqlx::Type<Db>>::type_info(),
            SqlValue::F64(_) => <f64 as sqlx::Type<Db>>::type_info(),
            SqlValue::I8(_) => <i8 as sqlx::Type<Db>>::type_info(),
            SqlValue::I16(_) => <i16 as sqlx::Type<Db>>::type_info(),
            SqlValue::I32(_) => <i32 as sqlx::Type<Db>>::type_info(),
            SqlValue::I64(_) => <i64 as sqlx::Type<Db>>::type_info(),
            SqlValue::String(_) => <String as sqlx::Type<Db>>::type_info(),
            SqlValue::Interval(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::Bytes(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::List(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::Array(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::NaiveDate(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValue::NaiveDateTime(_) => <chrono::NaiveDateTime as sqlx::Type<Db>>::type_info(),
            SqlValue::NaiveTime(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValue::Uuid(_) => <uuid::Uuid as sqlx::Type<Db>>::type_info(),
            SqlValue::Decimal(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::BigDecimal(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::Range(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
        })
    }
}

#[always_context]
impl sqlx::Type<Db> for SqlValue {
    fn type_info() -> <Db as sqlx::Database>::TypeInfo {
        //Overriden by Encode anyway
        <Vec<u8> as sqlx::Type<Db>>::type_info()
    }
}
#[derive(Debug)]
pub enum SqlValueMaybeRef<'a> {
    Ref(SqlValueRef<'a>),
    Value(SqlValue),
    Vec(Vec<SqlValueMaybeRef<'a>>),
    Option(Option<Box<SqlValueMaybeRef<'a>>>),
}

fn escape_sql(input: &str) -> String {
    let mut escaped = String::new();
    for character in input.chars() {
        match character {
            '\'' => escaped.push_str("''"), // Escape single quotes
            _ => escaped.push(character),   // Other characters remain unchanged
        }
    }
    escaped
}

#[always_context]
impl<'a> DriverValue<'a, super::Db> for SqlValueMaybeRef<'a> {
    fn to_default(&self) -> anyhow::Result<String> {
        Ok(match self {
            SqlValueMaybeRef::Ref(v) => match v {
                SqlValueRef::Bool(b) => b.to_string(),
                SqlValueRef::F32(f) => f.to_string(),
                SqlValueRef::F64(f) => f.to_string(),
                SqlValueRef::I8(i) => i.to_string(),
                SqlValueRef::I16(i) => i.to_string(),
                SqlValueRef::I32(i) => i.to_string(),
                SqlValueRef::I64(i) => i.to_string(),
                SqlValueRef::String(s) => format!("'{}'", escape_sql(s)),
                SqlValueRef::Str(s) => format!("'{}'", escape_sql(s)),
                SqlValueRef::NaiveDate(naive_date) => {
                    format!("'{}'", naive_date.format("%F"))
                }
                SqlValueRef::NaiveDateTime(naive_date_time) => {
                    format!("'{}'", naive_date_time.format("%F %T%.f"))
                }
                SqlValueRef::NaiveTime(naive_time) => format!("'{}'", naive_time.format("%T%.f")),
                _ => {
                    anyhow::bail!("Default value on binary (BLOB) types is not supported!");
                }
            },
            SqlValueMaybeRef::Value(v) => match v {
                SqlValue::Bool(v2) => v2.to_string(),
                SqlValue::F32(v2) => v2.to_string(),
                SqlValue::F64(v2) => v2.to_string(),
                SqlValue::I8(v2) => v2.to_string(),
                SqlValue::I16(v2) => v2.to_string(),
                SqlValue::I32(v2) => v2.to_string(),
                SqlValue::I64(v2) => v2.to_string(),
                SqlValue::String(s) => format!("'{}'", escape_sql(s)),
                SqlValue::NaiveDate(naive_date) => {
                    format!("'{}'", naive_date.format("%F"))
                }
                SqlValue::NaiveDateTime(naive_date_time) => {
                    format!("'{}'", naive_date_time.format("%F %T%.f"))
                }
                SqlValue::NaiveTime(naive_time) => format!("'{}'", naive_time.format("%T%.f")),
                _ => {
                    anyhow::bail!("Default value on binary (BLOB) types is not supported!");
                }
            },
            SqlValueMaybeRef::Vec(_) => {
                anyhow::bail!("Default value on binary (BLOB) types is not supported!");
            }
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    return v.to_default();
                } else {
                    "NULL".to_string()
                }
            }
        })
    }
}

#[always_context]
impl<'a> Encode<'a, Db> for SqlValueMaybeRef<'a> {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlValueMaybeRef::Ref(v) => v.encode_by_ref(buf),
            SqlValueMaybeRef::Value(v) => v.encode_by_ref(buf),
            SqlValueMaybeRef::Vec(v) => {
                <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    v.encode_by_ref(buf)
                } else {
                    Ok(sqlx::encode::IsNull::Yes)
                }
            }
        }
    }

    fn encode(
        self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        match self {
            SqlValueMaybeRef::Ref(v) => v.encode(buf),
            SqlValueMaybeRef::Value(v) => v.encode(buf),
            SqlValueMaybeRef::Vec(v) => <Vec<u8> as Encode<'a, Db>>::encode(binary(v)?, buf),
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    v.encode(buf)
                } else {
                    Ok(sqlx::encode::IsNull::Yes)
                }
            }
        }
    }

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        match self {
            SqlValueMaybeRef::Ref(v) => v.produces(),
            SqlValueMaybeRef::Value(v) => v.produces(),
            SqlValueMaybeRef::Vec(_) => Some(<Vec<u8> as sqlx::Type<Db>>::type_info()),
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    v.produces()
                } else {
                    None
                }
            }
        }
    }
}

#[always_context]
impl sqlx::Type<Db> for SqlValueMaybeRef<'_> {
    fn type_info() -> <Db as sqlx::Database>::TypeInfo {
        //Overriden by Encode anyway
        <Vec<u8> as sqlx::Type<Db>>::type_info()
    }
}

#[always_context]
impl serde::Serialize for SqlValueMaybeRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SqlValueMaybeRef::Ref(v) => {
                serializer.serialize_newtype_variant("SqlValueMaybeRef", 0, "Value", v)
            }
            SqlValueMaybeRef::Value(v) => {
                serializer.serialize_newtype_variant("SqlValueMaybeRef", 0, "Value", v)
            }
            SqlValueMaybeRef::Vec(v) => {
                serializer.serialize_newtype_variant("SqlValueMaybeRef", 1, "Vec", v)
            }
            SqlValueMaybeRef::Option(v) => {
                serializer.serialize_newtype_variant("SqlValueMaybeRef", 2, "Option", v)
            }
        }
    }
}

mod value_serialize {
    use super::always_context;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub enum SqlValueMaybeRef<'a> {
        Value(super::SqlValue),
        Vec(Vec<super::SqlValueMaybeRef<'a>>),
        Option(Option<Box<super::SqlValueMaybeRef<'a>>>),
    }

    #[always_context]
    impl<'a> From<SqlValueMaybeRef<'a>> for super::SqlValueMaybeRef<'a> {
        fn from(value: SqlValueMaybeRef<'a>) -> Self {
            match value {
                SqlValueMaybeRef::Value(v) => Self::Value(v),
                SqlValueMaybeRef::Vec(v) => Self::Vec(v),
                SqlValueMaybeRef::Option(v) => Self::Option(v),
            }
        }
    }
}

#[always_context]
impl<'b> serde::Deserialize<'b> for SqlValueMaybeRef<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'b>,
    {
        value_serialize::SqlValueMaybeRef::deserialize(deserializer).map(|v| v.into())
    }
}
// IpAddr
#[always_context]
impl<'a> From<&'a std::net::IpAddr> for SqlValueMaybeRef<'a> {
    fn from(value: &'a std::net::IpAddr) -> Self {
        Self::Ref(SqlValueRef::IpAddr(value))
    }
}
#[always_context]
impl From<std::net::IpAddr> for SqlValueMaybeRef<'_> {
    fn from(value: std::net::IpAddr) -> Self {
        Self::Value(SqlValue::IpAddr(value))
    }
}
// Bool
#[always_context]
impl<'a> From<&'a bool> for SqlValueMaybeRef<'a> {
    fn from(value: &'a bool) -> Self {
        Self::Ref(SqlValueRef::Bool(value))
    }
}
#[always_context]
impl From<bool> for SqlValueMaybeRef<'_> {
    fn from(value: bool) -> Self {
        Self::Value(SqlValue::Bool(value))
    }
}

// F32
#[always_context]
impl<'a> From<&'a f32> for SqlValueMaybeRef<'a> {
    fn from(value: &'a f32) -> Self {
        Self::Ref(SqlValueRef::F32(value))
    }
}
#[always_context]
impl From<f32> for SqlValueMaybeRef<'_> {
    fn from(value: f32) -> Self {
        Self::Value(SqlValue::F32(value))
    }
}
// F64
#[always_context]
impl<'a> From<&'a f64> for SqlValueMaybeRef<'a> {
    fn from(value: &'a f64) -> Self {
        Self::Ref(SqlValueRef::F64(value))
    }
}
#[always_context]
impl From<f64> for SqlValueMaybeRef<'_> {
    fn from(value: f64) -> Self {
        Self::Value(SqlValue::F64(value))
    }
}
// I8
#[always_context]
impl<'a> From<&'a i8> for SqlValueMaybeRef<'a> {
    fn from(value: &'a i8) -> Self {
        Self::Ref(SqlValueRef::I8(value))
    }
}
#[always_context]
impl From<i8> for SqlValueMaybeRef<'_> {
    fn from(value: i8) -> Self {
        Self::Value(SqlValue::I8(value))
    }
}
// I16
#[always_context]
impl<'a> From<&'a i16> for SqlValueMaybeRef<'a> {
    fn from(value: &'a i16) -> Self {
        Self::Ref(SqlValueRef::I16(value))
    }
}
#[always_context]
impl From<i16> for SqlValueMaybeRef<'_> {
    fn from(value: i16) -> Self {
        Self::Value(SqlValue::I16(value))
    }
}
// I32
#[always_context]
impl<'a> From<&'a i32> for SqlValueMaybeRef<'a> {
    fn from(value: &'a i32) -> Self {
        Self::Ref(SqlValueRef::I32(value))
    }
}
#[always_context]
impl From<i32> for SqlValueMaybeRef<'_> {
    fn from(value: i32) -> Self {
        Self::Value(SqlValue::I32(value))
    }
}
// I64
#[always_context]
impl<'a> From<&'a i64> for SqlValueMaybeRef<'a> {
    fn from(value: &'a i64) -> Self {
        Self::Ref(SqlValueRef::I64(value))
    }
}
#[always_context]
impl From<i64> for SqlValueMaybeRef<'_> {
    fn from(value: i64) -> Self {
        Self::Value(SqlValue::I64(value))
    }
}
// String
#[always_context]
impl<'a> From<&'a str> for SqlValueMaybeRef<'a> {
    fn from(value: &'a str) -> Self {
        Self::Ref(SqlValueRef::Str(value))
    }
}
#[always_context]
impl<'a> From<&'a String> for SqlValueMaybeRef<'a> {
    fn from(value: &'a String) -> Self {
        Self::Ref(SqlValueRef::String(value))
    }
}
#[always_context]
impl From<String> for SqlValueMaybeRef<'_> {
    fn from(value: String) -> Self {
        Self::Value(SqlValue::String(value))
    }
}
// Interval
#[always_context]
impl<'a> From<&'a PgInterval> for SqlValueMaybeRef<'a> {
    fn from(value: &'a PgInterval) -> Self {
        Self::Ref(SqlValueRef::Interval(value))
    }
}
#[always_context]
impl From<PgInterval> for SqlValueMaybeRef<'_> {
    fn from(value: PgInterval) -> Self {
        Self::Value(SqlValue::Interval(value))
    }
}
// Bytes
#[always_context]
impl<'a> From<&'a Vec<u8>> for SqlValueMaybeRef<'a> {
    fn from(value: &'a Vec<u8>) -> Self {
        Self::Ref(SqlValueRef::Bytes(value))
    }
}
#[always_context]
impl From<Vec<u8>> for SqlValueMaybeRef<'_> {
    fn from(value: Vec<u8>) -> Self {
        Self::Value(SqlValue::Bytes(value))
    }
}
// List
/* #[always_context]
impl<'a, T> From<&'a Vec<T>> for SqlValueMaybeRef<'a>
where
    &'a T: Into<SqlValueMaybeRef<'a>>,
{
    fn from(value: &'a Vec<T>) -> Self {
        let mut v = Vec::new();
        for i in value.iter() {
            v.push(i.into());
        }
        SqlValueMaybeRef::Vec(v)
    }
} */
#[always_context]
impl<'a, T: Into<SqlValueMaybeRef<'a>>> From<Vec<T>> for SqlValueMaybeRef<'a> {
    fn from(value: Vec<T>) -> Self {
        let mut v = Vec::new();
        for i in value.into_iter() {
            v.push(i.into());
        }
        SqlValueMaybeRef::Vec(v)
    }
}

#[always_context]
impl<'a, T: 'a> From<&'a Vec<T>> for SqlValueMaybeRef<'a>
where
    &'a T: Into<SqlValueMaybeRef<'a>>,
{
    fn from(value: &'a Vec<T>) -> Self {
        let mut v = Vec::new();
        for i in value.iter() {
            v.push(i.into());
        }
        SqlValueMaybeRef::Vec(v)
    }
}

// NaiveDate
#[always_context]
impl<'a> From<&'a chrono::NaiveDate> for SqlValueMaybeRef<'a> {
    fn from(value: &'a chrono::NaiveDate) -> Self {
        Self::Ref(SqlValueRef::NaiveDate(value))
    }
}
#[always_context]
impl From<chrono::NaiveDate> for SqlValueMaybeRef<'_> {
    fn from(value: chrono::NaiveDate) -> Self {
        Self::Value(SqlValue::NaiveDate(value))
    }
}
// NaiveDateTime
#[always_context]
impl<'a> From<&'a chrono::NaiveDateTime> for SqlValueMaybeRef<'a> {
    fn from(value: &'a chrono::NaiveDateTime) -> Self {
        Self::Ref(SqlValueRef::NaiveDateTime(value))
    }
}
#[always_context]
impl From<chrono::NaiveDateTime> for SqlValueMaybeRef<'_> {
    fn from(value: chrono::NaiveDateTime) -> Self {
        Self::Value(SqlValue::NaiveDateTime(value))
    }
}
// NaiveTime
#[always_context]
impl<'a> From<&'a chrono::NaiveTime> for SqlValueMaybeRef<'a> {
    fn from(value: &'a chrono::NaiveTime) -> Self {
        Self::Ref(SqlValueRef::NaiveTime(value))
    }
}
#[always_context]
impl From<chrono::NaiveTime> for SqlValueMaybeRef<'_> {
    fn from(value: chrono::NaiveTime) -> Self {
        Self::Value(SqlValue::NaiveTime(value))
    }
}
// Uuid
#[always_context]
impl<'a> From<&'a uuid::Uuid> for SqlValueMaybeRef<'a> {
    fn from(value: &'a uuid::Uuid) -> Self {
        Self::Ref(SqlValueRef::Uuid(value))
    }
}
#[always_context]
impl From<uuid::Uuid> for SqlValueMaybeRef<'_> {
    fn from(value: uuid::Uuid) -> Self {
        Self::Value(SqlValue::Uuid(value))
    }
}
// Decimal
#[always_context]
impl<'a> From<&'a sqlx::types::Decimal> for SqlValueMaybeRef<'a> {
    fn from(value: &'a sqlx::types::Decimal) -> Self {
        Self::Ref(SqlValueRef::Decimal(value))
    }
}
#[always_context]
impl From<sqlx::types::Decimal> for SqlValueMaybeRef<'_> {
    fn from(value: sqlx::types::Decimal) -> Self {
        Self::Value(SqlValue::Decimal(value))
    }
}
// BigDecimal
#[always_context]
impl<'a> From<&'a sqlx::types::BigDecimal> for SqlValueMaybeRef<'a> {
    fn from(value: &'a sqlx::types::BigDecimal) -> Self {
        Self::Ref(SqlValueRef::BigDecimal(value))
    }
}
#[always_context]
impl From<sqlx::types::BigDecimal> for SqlValueMaybeRef<'_> {
    fn from(value: sqlx::types::BigDecimal) -> Self {
        Self::Value(SqlValue::BigDecimal(value))
    }
}
// Range
#[always_context]
impl From<std::ops::Range<i32>> for SqlValueMaybeRef<'_> {
    fn from(value: std::ops::Range<i32>) -> Self {
        Self::Value(SqlValue::Range(SqlRangeValue::I32(value.into())))
    }
}
#[always_context]
impl From<std::ops::Range<i64>> for SqlValueMaybeRef<'_> {
    fn from(value: std::ops::Range<i64>) -> Self {
        Self::Value(SqlValue::Range(SqlRangeValue::I64(value.into())))
    }
}
#[always_context]
impl From<std::ops::Range<NaiveDate>> for SqlValueMaybeRef<'_> {
    fn from(value: std::ops::Range<NaiveDate>) -> Self {
        Self::Value(SqlValue::Range(SqlRangeValue::NaiveDate(value.into())))
    }
}
#[always_context]
impl From<std::ops::Range<NaiveDateTime>> for SqlValueMaybeRef<'_> {
    fn from(value: std::ops::Range<NaiveDateTime>) -> Self {
        Self::Value(SqlValue::Range(SqlRangeValue::NaiveDateTime(value.into())))
    }
}
#[always_context]
impl From<std::ops::Range<Decimal>> for SqlValueMaybeRef<'_> {
    fn from(value: std::ops::Range<Decimal>) -> Self {
        Self::Value(SqlValue::Range(SqlRangeValue::Decimal(value.into())))
    }
}
#[always_context]
impl From<std::ops::Range<BigDecimal>> for SqlValueMaybeRef<'_> {
    fn from(value: std::ops::Range<BigDecimal>) -> Self {
        Self::Value(SqlValue::Range(SqlRangeValue::BigDecimal(value.into())))
    }
}

//Option
#[always_context]
impl<'a, T: Into<SqlValueMaybeRef<'a>>> From<Option<T>> for SqlValueMaybeRef<'a> {
    fn from(value: Option<T>) -> Self {
        match value {
            Some(v) => SqlValueMaybeRef::Option(Some(Box::new(v.into()))),
            None => SqlValueMaybeRef::Option(None),
        }
    }
}

#[always_context]
impl<'a, T: 'a> From<&'a Option<T>> for SqlValueMaybeRef<'a>
where
    &'a T: Into<SqlValueMaybeRef<'a>>,
{
    fn from(value: &'a Option<T>) -> Self {
        match value {
            Some(v) => SqlValueMaybeRef::Option(Some(Box::new(v.into()))),
            None => SqlValueMaybeRef::Option(None),
        }
    }
}
