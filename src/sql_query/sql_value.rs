use std::ops::Bound;

use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};
use sqlx::{
    Encode,
    postgres::types::{PgInterval, PgRange},
};

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
impl<'a> Encode<'a, crate::Db> for SqlValueRef<'a> {
    fn encode_by_ref(
        &self,
        buf: &mut <crate::Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlValueRef::IpAddr(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueRef::Bool(v) => <bool as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::F32(v) => <f32 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::F64(v) => <f64 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::I8(v) => <i8 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::I16(v) => <i16 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::I32(v) => <i32 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::I64(v) => <i64 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::String(v) => <String as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::Interval(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(
                &binary(PgIntervalSerde2::from(*v))?,
                buf,
            ),
            SqlValueRef::Bytes(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::List(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueRef::Array(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueRef::NaiveDate(v) => {
                <chrono::NaiveDate as Encode<'a, crate::Db>>::encode_by_ref(v, buf)
            }
            SqlValueRef::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, crate::Db>>::encode_by_ref(v, buf)
            }
            SqlValueRef::NaiveTime(v) => {
                <chrono::NaiveTime as Encode<'a, crate::Db>>::encode_by_ref(v, buf)
            }
            SqlValueRef::Uuid(v) => <uuid::Uuid as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValueRef::Decimal(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueRef::BigDecimal(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueRef::Range(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
        }
    }

    /* fn encode(
        self,
        buf: &mut <crate::Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        match self {
            SqlValue::IpAddr(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::String(v) => <String as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::Interval(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(PgIntervalSerde2::from(v))?, buf)
            }
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::List(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::Array(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, crate::Db>>::encode(v, buf)
            }
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::Decimal(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::BigDecimal(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::Range(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
        }
    } */

    fn produces(&self) -> Option<<crate::Db as sqlx::Database>::TypeInfo> {
        Some(match self {
            SqlValueRef::IpAddr(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Bool(_) => <bool as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::F32(_) => <f32 as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::F64(_) => <f64 as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::I8(_) => <i8 as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::I16(_) => <i16 as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::I32(_) => <i32 as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::I64(_) => <i64 as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::String(_) => <String as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Interval(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Bytes(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::List(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Array(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::NaiveDate(_) => <chrono::NaiveDate as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::NaiveDateTime(_) => {
                <chrono::NaiveDateTime as sqlx::Type<crate::Db>>::type_info()
            }
            SqlValueRef::NaiveTime(_) => <chrono::NaiveDate as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Uuid(_) => <uuid::Uuid as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Decimal(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::BigDecimal(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValueRef::Range(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
        })
    }
}

#[always_context]
impl sqlx::Type<crate::Db> for SqlValueRef<'_> {
    fn type_info() -> <crate::Db as sqlx::Database>::TypeInfo {
        //Overriden by Encode anyway
        <Vec<u8> as sqlx::Type<crate::Db>>::type_info()
    }
}

#[always_context]
impl<'a> Encode<'a, crate::Db> for SqlValue {
    fn encode_by_ref(
        &self,
        buf: &mut <crate::Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlValue::IpAddr(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValue::Bool(v) => <bool as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::String(v) => <String as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::Interval(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(
                &binary(PgIntervalSerde2::from(*v))?,
                buf,
            ),
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::List(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValue::Array(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValue::NaiveDate(v) => {
                <chrono::NaiveDate as Encode<'a, crate::Db>>::encode_by_ref(v, buf)
            }
            SqlValue::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, crate::Db>>::encode_by_ref(v, buf)
            }
            SqlValue::NaiveTime(v) => {
                <chrono::NaiveTime as Encode<'a, crate::Db>>::encode_by_ref(v, buf)
            }
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, crate::Db>>::encode_by_ref(v, buf),
            SqlValue::Decimal(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValue::BigDecimal(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValue::Range(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode_by_ref(&binary(v)?, buf)
            }
        }
    }

    fn encode(
        self,
        buf: &mut <crate::Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        match self {
            SqlValue::IpAddr(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::String(v) => <String as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::Interval(v) => {
                <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(PgIntervalSerde2::from(v))?, buf)
            }
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::List(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::Array(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, crate::Db>>::encode(v, buf)
            }
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, crate::Db>>::encode(v, buf),
            SqlValue::Decimal(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::BigDecimal(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
            SqlValue::Range(v) => <Vec<u8> as Encode<'a, crate::Db>>::encode(binary(v)?, buf),
        }
    }

    fn produces(&self) -> Option<<crate::Db as sqlx::Database>::TypeInfo> {
        Some(match self {
            SqlValue::IpAddr(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Bool(_) => <bool as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::F32(_) => <f32 as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::F64(_) => <f64 as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::I8(_) => <i8 as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::I16(_) => <i16 as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::I32(_) => <i32 as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::I64(_) => <i64 as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::String(_) => <String as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Interval(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Bytes(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::List(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Array(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::NaiveDate(_) => <chrono::NaiveDate as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::NaiveDateTime(_) => {
                <chrono::NaiveDateTime as sqlx::Type<crate::Db>>::type_info()
            }
            SqlValue::NaiveTime(_) => <chrono::NaiveDate as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Uuid(_) => <uuid::Uuid as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Decimal(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::BigDecimal(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
            SqlValue::Range(_) => <Vec<u8> as sqlx::Type<crate::Db>>::type_info(),
        })
    }
}

#[always_context]
impl sqlx::Type<crate::Db> for SqlValue {
    fn type_info() -> <crate::Db as sqlx::Database>::TypeInfo {
        //Overriden by Encode anyway
        <Vec<u8> as sqlx::Type<crate::Db>>::type_info()
    }
}
#[derive(Debug)]
pub enum SqlValueMaybeRef<'a> {
    Ref(SqlValueRef<'a>),
    Value(SqlValue),
}

#[always_context]
impl<'a> Encode<'a, crate::Db> for SqlValueMaybeRef<'a> {
    fn encode_by_ref(
        &self,
        buf: &mut <crate::Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlValueMaybeRef::Ref(v) => v.encode_by_ref(buf),
            SqlValueMaybeRef::Value(v) => v.encode_by_ref(buf),
        }
    }

    fn encode(
        self,
        buf: &mut <crate::Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        match self {
            SqlValueMaybeRef::Ref(v) => v.encode(buf),
            SqlValueMaybeRef::Value(v) => v.encode(buf),
        }
    }

    fn produces(&self) -> Option<<crate::Db as sqlx::Database>::TypeInfo> {
        match self {
            SqlValueMaybeRef::Ref(v) => v.produces(),
            SqlValueMaybeRef::Value(v) => v.produces(),
        }
    }
}

#[always_context]
impl sqlx::Type<crate::Db> for SqlValueMaybeRef<'_> {
    fn type_info() -> <crate::Db as sqlx::Database>::TypeInfo {
        //Overriden by Encode anyway
        <Vec<u8> as sqlx::Type<crate::Db>>::type_info()
    }
}

#[always_context]
impl serde::Serialize for SqlValueMaybeRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SqlValueMaybeRef::Ref(v) => v.serialize(serializer),
            SqlValueMaybeRef::Value(v) => v.serialize(serializer),
        }
    }
}

#[always_context]
impl<'b> serde::Deserialize<'b> for SqlValueMaybeRef<'_> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'b>,
    {
        let value = SqlValue::deserialize(deserializer)?;
        Ok(SqlValueMaybeRef::Value(value))
    }
}
