
#[cfg(feature = "ipnet")]
use std::net::IpAddr;

use bigdecimal::BigDecimal;
use easy_macros::macros::always_context;
use rust_decimal::Decimal;
use sqlx::{
    Encode,
    postgres::types::{PgInterval, PgRange},
};

use crate::{
    DriverValue,
    general_value::{SqlRangeValue, SqlRangeValueRef, SqlValue, SqlValueMaybeRef, SqlValueRef},
};

use super::Db;

fn binary<T: serde::Serialize>(v: T) -> Result<Vec<u8>, bincode::error::EncodeError> {
    bincode::serde::encode_to_vec(v, bincode::config::standard())
}

#[always_context]
impl<'a> Encode<'a, Db> for SqlValueRef<'a> {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            #[cfg(feature = "ipnet")]
            SqlValueRef::IpAddr(v) => <IpAddr as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Bool(v) => <bool as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::F32(v) => <f32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::F64(v) => <f64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I8(v) => <i8 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I16(v) => <i16 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I32(v) => <i32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::I64(v) => <i64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::String(v) => <String as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Str(v) => <&str as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Interval(v) => <PgInterval as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::List(v) => <Vec<SqlValueRef<'a>> as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Array(v) => {
                <Vec<SqlValueRef<'a>> as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
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
            SqlValueRef::Decimal(v) => <Decimal as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::BigDecimal(v) => <BigDecimal as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueRef::Range(v) => {
                <SqlRangeValueRef<'a> as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
        }
    }

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        Some(match self {
            #[cfg(feature = "ipnet")]
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
            SqlValueRef::Interval(_) => <PgInterval as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Bytes(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            //TODO Fix this, try Encodeable approach again
            SqlValueRef::List(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            //TODO Fix this, try Encodeable approach again
            SqlValueRef::Array(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValueRef::NaiveDate(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValueRef::NaiveDateTime(_) => <chrono::NaiveDateTime as sqlx::Type<Db>>::type_info(),
            SqlValueRef::NaiveTime(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Uuid(_) => <uuid::Uuid as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Decimal(_) => <Decimal as sqlx::Type<Db>>::type_info(),
            SqlValueRef::BigDecimal(_) => <BigDecimal as sqlx::Type<Db>>::type_info(),
            SqlValueRef::Range(v) => return <SqlRangeValueRef<'_> as Encode<'_, Db>>::produces(v),
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

impl<'a> Encode<'a, Db> for SqlRangeValue {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlRangeValue::I32(pg_range) => {
                <PgRange<i32> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValue::I64(pg_range) => {
                <PgRange<i64> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValue::NaiveDate(pg_range) => {
                <PgRange<chrono::NaiveDate> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValue::NaiveDateTime(pg_range) => {
                <PgRange<chrono::NaiveDateTime> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValue::BigDecimal(pg_range) => {
                <PgRange<BigDecimal> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValue::Decimal(pg_range) => {
                <PgRange<Decimal> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
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
            SqlRangeValue::I32(pg_range) => <PgRange<i32> as Encode<'a, Db>>::encode(pg_range, buf),
            SqlRangeValue::I64(pg_range) => <PgRange<i64> as Encode<'a, Db>>::encode(pg_range, buf),
            SqlRangeValue::NaiveDate(pg_range) => {
                <PgRange<chrono::NaiveDate> as Encode<'a, Db>>::encode(pg_range, buf)
            }
            SqlRangeValue::NaiveDateTime(pg_range) => {
                <PgRange<chrono::NaiveDateTime> as Encode<'a, Db>>::encode(pg_range, buf)
            }
            SqlRangeValue::BigDecimal(pg_range) => {
                <PgRange<BigDecimal> as Encode<'a, Db>>::encode(pg_range, buf)
            }
            SqlRangeValue::Decimal(pg_range) => {
                <PgRange<Decimal> as Encode<'a, Db>>::encode(pg_range, buf)
            }
        }
    }

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        match self {
            SqlRangeValue::I32(_) => Some(<PgRange<i32> as sqlx::Type<Db>>::type_info()),
            SqlRangeValue::I64(_) => Some(<PgRange<i64> as sqlx::Type<Db>>::type_info()),
            SqlRangeValue::NaiveDate(_) => {
                Some(<PgRange<chrono::NaiveDate> as sqlx::Type<Db>>::type_info())
            }
            SqlRangeValue::NaiveDateTime(_) => {
                Some(<PgRange<chrono::NaiveDateTime> as sqlx::Type<Db>>::type_info())
            }
            SqlRangeValue::BigDecimal(_) => {
                Some(<PgRange<BigDecimal> as sqlx::Type<Db>>::type_info())
            }
            SqlRangeValue::Decimal(_) => Some(<PgRange<Decimal> as sqlx::Type<Db>>::type_info()),
        }
    }
}

impl<'a> Encode<'a, Db> for SqlRangeValueRef<'a> {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            SqlRangeValueRef::I32(pg_range) => {
                <PgRange<i32> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValueRef::I64(pg_range) => {
                <PgRange<i64> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValueRef::NaiveDate(pg_range) => {
                <PgRange<chrono::NaiveDate> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValueRef::NaiveDateTime(pg_range) => {
                <PgRange<chrono::NaiveDateTime> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValueRef::BigDecimal(pg_range) => {
                <PgRange<BigDecimal> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
            SqlRangeValueRef::Decimal(pg_range) => {
                <PgRange<Decimal> as Encode<'a, Db>>::encode_by_ref(pg_range, buf)
            }
        }
    }
    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        match self {
            SqlRangeValueRef::I32(_) => Some(<PgRange<i32> as sqlx::Type<Db>>::type_info()),
            SqlRangeValueRef::I64(_) => Some(<PgRange<i64> as sqlx::Type<Db>>::type_info()),
            SqlRangeValueRef::NaiveDate(_) => {
                Some(<PgRange<chrono::NaiveDate> as sqlx::Type<Db>>::type_info())
            }
            SqlRangeValueRef::NaiveDateTime(_) => {
                Some(<PgRange<chrono::NaiveDateTime> as sqlx::Type<Db>>::type_info())
            }
            SqlRangeValueRef::BigDecimal(_) => {
                Some(<PgRange<BigDecimal> as sqlx::Type<Db>>::type_info())
            }
            SqlRangeValueRef::Decimal(_) => Some(<PgRange<Decimal> as sqlx::Type<Db>>::type_info()),
        }
    }
}

#[always_context]
impl<'a> Encode<'a, Db> for SqlValue {
    fn encode_by_ref(
        &self,
        buf: &mut <Db as sqlx::Database>::ArgumentBuffer<'a>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        match self {
            #[cfg(feature = "ipnet")]
            SqlValue::IpAddr(v) => <IpAddr as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::String(v) => <String as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Interval(v) => <PgInterval as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(v, buf),
            //TODO Fix this, try Encodeable approach again
            SqlValue::List(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            //TODO Fix this, try Encodeable approach again
            SqlValue::Array(v) => <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::NaiveDateTime(v) => {
                <chrono::NaiveDateTime as Encode<'a, Db>>::encode_by_ref(v, buf)
            }
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Decimal(v) => <Decimal as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::BigDecimal(v) => <BigDecimal as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValue::Range(v) => <SqlRangeValue as Encode<'a, Db>>::encode_by_ref(v, buf),
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
            #[cfg(feature = "ipnet")]
            SqlValue::IpAddr(v) => <IpAddr as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Bool(v) => <bool as Encode<'a, Db>>::encode(v, buf),
            SqlValue::F32(v) => <f32 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::F64(v) => <f64 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I8(v) => <i8 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I16(v) => <i16 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I32(v) => <i32 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::I64(v) => <i64 as Encode<'a, Db>>::encode(v, buf),
            SqlValue::String(v) => <String as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Interval(v) => <PgInterval as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Bytes(v) => <Vec<u8> as Encode<'a, Db>>::encode(v, buf),
            SqlValue::List(v) => <Vec<SqlValue> as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Array(v) => <Vec<SqlValue> as Encode<'a, Db>>::encode(v, buf),
            SqlValue::NaiveDate(v) => <chrono::NaiveDate as Encode<'a, Db>>::encode(v, buf),
            SqlValue::NaiveDateTime(v) => <chrono::NaiveDateTime as Encode<'a, Db>>::encode(v, buf),
            SqlValue::NaiveTime(v) => <chrono::NaiveTime as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Uuid(v) => <uuid::Uuid as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Decimal(v) => <Decimal as Encode<'a, Db>>::encode(v, buf),
            SqlValue::BigDecimal(v) => <BigDecimal as Encode<'a, Db>>::encode(v, buf),
            SqlValue::Range(v) => <SqlRangeValue as Encode<'a, Db>>::encode(v, buf),
        }
    }

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        Some(match self {
            #[cfg(feature = "ipnet")]
            SqlValue::IpAddr(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::Bool(_) => <bool as sqlx::Type<Db>>::type_info(),
            SqlValue::F32(_) => <f32 as sqlx::Type<Db>>::type_info(),
            SqlValue::F64(_) => <f64 as sqlx::Type<Db>>::type_info(),
            SqlValue::I8(_) => <i8 as sqlx::Type<Db>>::type_info(),
            SqlValue::I16(_) => <i16 as sqlx::Type<Db>>::type_info(),
            SqlValue::I32(_) => <i32 as sqlx::Type<Db>>::type_info(),
            SqlValue::I64(_) => <i64 as sqlx::Type<Db>>::type_info(),
            SqlValue::String(_) => <String as sqlx::Type<Db>>::type_info(),
            SqlValue::Interval(_) => <PgInterval as sqlx::Type<Db>>::type_info(),
            SqlValue::Bytes(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::List(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::Array(_) => <Vec<u8> as sqlx::Type<Db>>::type_info(),
            SqlValue::NaiveDate(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValue::NaiveDateTime(_) => <chrono::NaiveDateTime as sqlx::Type<Db>>::type_info(),
            SqlValue::NaiveTime(_) => <chrono::NaiveDate as sqlx::Type<Db>>::type_info(),
            SqlValue::Uuid(_) => <uuid::Uuid as sqlx::Type<Db>>::type_info(),
            SqlValue::Decimal(_) => <Decimal as sqlx::Type<Db>>::type_info(),
            SqlValue::BigDecimal(_) => <BigDecimal as sqlx::Type<Db>>::type_info(),
            SqlValue::Range(v) => return <SqlRangeValue as Encode<'_, Db>>::produces(v),
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

fn escape_sql(input: &str) -> String {
    let mut escaped = String::new();
    for character in input.chars() {
        match character {
            '\'' => escaped.push_str("''"), // Escape single quotes (standard SQL)
            '\\' => escaped.push_str("\\\\"), // Escape backslashes for safety
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
                #[cfg(feature = "ipnet")]
                SqlValueRef::IpAddr(ip) => format!("'{}'::inet", escape_sql(&ip.to_string())),
                SqlValueRef::Bool(b) => b.to_string().to_uppercase(),
                SqlValueRef::F32(f) => f.to_string(),
                SqlValueRef::F64(f) => f.to_string(),
                SqlValueRef::I8(i) => i.to_string(),
                SqlValueRef::I16(i) => i.to_string(),
                SqlValueRef::I32(i) => i.to_string(),
                SqlValueRef::I64(i) => i.to_string(),
                SqlValueRef::String(s) => format!("'{}'", escape_sql(s)),
                SqlValueRef::Str(s) => format!("'{}'", escape_sql(s)),
                SqlValueRef::Interval(interval) => {
                    // PostgreSQL interval format: 'P{months}M{days}DT{hours}H{minutes}M{seconds}S'::interval
                    // Or more simply: '{days} days {microseconds} microseconds {months} months'::interval
                    format!(
                        "'{} days {} months {} microseconds'::interval",
                        interval.days, interval.months, interval.microseconds
                    )
                }
                SqlValueRef::NaiveDate(naive_date) => {
                    format!("'{}'::date", naive_date.format("%Y-%m-%d"))
                }
                SqlValueRef::NaiveDateTime(naive_date_time) => {
                    format!(
                        "'{}'::timestamp",
                        naive_date_time.format("%Y-%m-%d %H:%M:%S%.f")
                    )
                }
                SqlValueRef::NaiveTime(naive_time) => {
                    format!("'{}'::time", naive_time.format("%H:%M:%S%.f"))
                }
                SqlValueRef::Uuid(uuid) => {
                    format!("'{uuid}'::uuid",)
                }
                SqlValueRef::Decimal(d) => {
                    format!("{d}::numeric",)
                }
                SqlValueRef::BigDecimal(bd) => {
                    format!("{bd}::numeric",)
                }
                SqlValueRef::Bytes(bytes) => {
                    let hex_string = bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    format!("'\\x{}'::bytea", hex_string)
                }
                SqlValueRef::List(_) | SqlValueRef::Array(_) => {
                    anyhow::bail!("Default value on array types is not supported!");
                }
                SqlValueRef::Range(_) => {
                    anyhow::bail!("Default value on range types is not supported!");
                }
            },
            SqlValueMaybeRef::Value(v) => match v {
                #[cfg(feature = "ipnet")]
                SqlValue::IpAddr(ip) => format!("'{}'::inet", escape_sql(&ip.to_string())),
                SqlValue::Bool(v2) => v2.to_string().to_uppercase(),
                SqlValue::F32(v2) => v2.to_string(),
                SqlValue::F64(v2) => v2.to_string(),
                SqlValue::I8(v2) => v2.to_string(),
                SqlValue::I16(v2) => v2.to_string(),
                SqlValue::I32(v2) => v2.to_string(),
                SqlValue::I64(v2) => v2.to_string(),
                SqlValue::String(s) => format!("'{}'", escape_sql(s)),
                SqlValue::Interval(interval) => {
                    format!(
                        "'{} days {} months {} microseconds'::interval",
                        interval.days, interval.months, interval.microseconds
                    )
                }
                SqlValue::NaiveDate(naive_date) => {
                    format!("'{}'::date", naive_date.format("%Y-%m-%d"))
                }
                SqlValue::NaiveDateTime(naive_date_time) => {
                    format!(
                        "'{}'::timestamp",
                        naive_date_time.format("%Y-%m-%d %H:%M:%S%.f")
                    )
                }
                SqlValue::NaiveTime(naive_time) => {
                    format!("'{}'::time", naive_time.format("%H:%M:%S%.f"))
                }
                SqlValue::Uuid(uuid) => {
                    format!("'{uuid}'::uuid")
                }
                SqlValue::Decimal(d) => {
                    format!("{d}::numeric",)
                }
                SqlValue::BigDecimal(bd) => {
                    format!("{bd}::numeric",)
                }
                SqlValue::Bytes(bytes) => {
                    let hex_string = bytes
                        .iter()
                        .map(|b| format!("{:02x}", b))
                        .collect::<String>();
                    format!("'\\x{}'::bytea", hex_string)
                }
                SqlValue::List(_) | SqlValue::Array(_) => {
                    anyhow::bail!("Default value on array types is not supported!");
                }
                SqlValue::Range(_) => {
                    anyhow::bail!("Default value on range types is not supported!");
                }
            },
            SqlValueMaybeRef::Vec(_) => {
                anyhow::bail!("Default value on array types is not supported!");
            }
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    return <SqlValueMaybeRef<'_> as DriverValue<'_, Db>>::to_default(v);
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
            SqlValueMaybeRef::Ref(v) => <SqlValueRef as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueMaybeRef::Value(v) => <SqlValue as Encode<'a, Db>>::encode_by_ref(v, buf),
            SqlValueMaybeRef::Vec(v) => {
                <Vec<u8> as Encode<'a, Db>>::encode_by_ref(&binary(v)?, buf)
            }
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    <SqlValueMaybeRef<'_> as Encode<'a, Db>>::encode_by_ref(v, buf)
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
            SqlValueMaybeRef::Ref(v) => <SqlValueRef as Encode<'a, Db>>::encode(v, buf),
            SqlValueMaybeRef::Value(v) => <SqlValue as Encode<'a, Db>>::encode(v, buf),
            SqlValueMaybeRef::Vec(v) => {
                <Vec<SqlValueMaybeRef<'a>> as Encode<'a, Db>>::encode(v, buf)
            }
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    <SqlValueMaybeRef<'a> as Encode<'a, Db>>::encode(*v, buf)
                } else {
                    Ok(sqlx::encode::IsNull::Yes)
                }
            }
        }
    }

    fn produces(&self) -> Option<<Db as sqlx::Database>::TypeInfo> {
        match self {
            SqlValueMaybeRef::Ref(v) => <SqlValueRef<'_> as Encode<'_, Db>>::produces(v),
            SqlValueMaybeRef::Value(v) => <SqlValue as Encode<'_, Db>>::produces(v),
            SqlValueMaybeRef::Vec(v) => {
                Some(<Vec<SqlValueMaybeRef<'_>> as Encode<'_, Db>>::produces(v)?)
            }
            SqlValueMaybeRef::Option(v) => {
                if let Some(v) = v {
                    <SqlValueMaybeRef<'_> as Encode<'_, Db>>::produces(v)
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
