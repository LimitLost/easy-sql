use std::{borrow::Cow, time::Duration};

use pg_escape::quote_literal;
use sqlx::postgres::types::PgInterval;

use crate::{ToDefault, impl_to_default_to_string_with_ref};

type D = super::Postgres;

#[cfg(feature = "ipnet")]
impl ToDefault<D> for sqlx::types::ipnet::IpNet {
    fn to_default(self) -> String {
        format!("'{}'::inet", self)
    }
}
#[cfg(feature = "ipnet")]
impl ToDefault<D> for std::net::IpAddr {
    fn to_default(self) -> String {
        format!("'{}'::inet", self)
    }
}

impl_to_default_to_string_with_ref!(bool);
impl_to_default_to_string_with_ref!(f32);
impl_to_default_to_string_with_ref!(f64);
impl_to_default_to_string_with_ref!(i8);
impl_to_default_to_string_with_ref!(i16);
impl_to_default_to_string_with_ref!(i32);
impl_to_default_to_string_with_ref!(i64);

impl ToDefault<D> for Cow<'_, str> {
    fn to_default(self) -> String {
        quote_literal(&self)
    }
}

impl ToDefault<D> for String {
    fn to_default(self) -> String {
        quote_literal(&self)
    }
}

impl ToDefault<D> for &String {
    fn to_default(self) -> String {
        quote_literal(self)
    }
}

impl ToDefault<D> for &str {
    fn to_default(self) -> String {
        quote_literal(self)
    }
}

impl ToDefault<D> for Duration {
    fn to_default(self) -> String {
        format!("'{} microseconds'::interval", self.as_micros())
    }
}

impl ToDefault<D> for &Duration {
    fn to_default(self) -> String {
        format!("'{} microseconds'::interval", self.as_micros())
    }
}

impl ToDefault<D> for PgInterval {
    fn to_default(self) -> String {
        format!(
            "'{} months {} days {} microseconds'::interval",
            self.months, self.days, self.microseconds
        )
    }
}

impl ToDefault<D> for &PgInterval {
    fn to_default(self) -> String {
        format!(
            "'{} months {} days {} microseconds'::interval",
            self.months, self.days, self.microseconds
        )
    }
}

#[cfg(feature = "bigdecimal")]
impl ToDefault<D> for bigdecimal::BigDecimal {
    fn to_default(self) -> String {
        format!("{self}::numeric",)
    }
}

#[cfg(feature = "bigdecimal")]
impl ToDefault<D> for &bigdecimal::BigDecimal {
    fn to_default(self) -> String {
        format!("{self}::numeric",)
    }
}
#[cfg(feature = "rust_decimal")]
impl ToDefault<D> for rust_decimal::Decimal {
    fn to_default(self) -> String {
        format!("{self}::numeric",)
    }
}

#[cfg(feature = "rust_decimal")]
impl ToDefault<D> for &rust_decimal::Decimal {
    fn to_default(self) -> String {
        format!("{self}::numeric",)
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::TimeDelta {
    fn to_default(self) -> String {
        format!(
            "'{} seconds {} microseconds'::interval",
            self.num_seconds(),
            self.subsec_nanos() / 1000
        )
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::TimeDelta {
    fn to_default(self) -> String {
        format!(
            "'{} seconds {} microseconds'::interval",
            self.num_seconds(),
            self.subsec_nanos() / 1000
        )
    }
}

impl ToDefault<D> for Vec<u8> {
    fn to_default(self) -> String {
        let hex_string = self
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        format!("'\\x{}'::bytea", hex_string)
    }
}
impl ToDefault<D> for &Vec<u8> {
    fn to_default(self) -> String {
        let hex_string = self
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        format!("'\\x{}'::bytea", hex_string)
    }
}

impl ToDefault<D> for &[u8] {
    fn to_default(self) -> String {
        let hex_string = self
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        format!("'\\x{}'::bytea", hex_string)
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::NaiveDate {
    fn to_default(self) -> String {
        format!("'{}'::date", self.format("%Y-%m-%d"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::NaiveDate {
    fn to_default(self) -> String {
        format!("'{}'::date", self.format("%Y-%m-%d"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::NaiveDateTime {
    fn to_default(self) -> String {
        format!("'{}'::timestamp", self.format("%Y-%m-%d %H:%M:%S%.f"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::NaiveDateTime {
    fn to_default(self) -> String {
        format!("'{}'::timestamp", self.format("%Y-%m-%d %H:%M:%S%.f"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::NaiveTime {
    fn to_default(self) -> String {
        format!("'{}'::time", self.format("%H:%M:%S%.f"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::NaiveTime {
    fn to_default(self) -> String {
        format!("'{}'::time", self.format("%H:%M:%S%.f"))
    }
}
#[cfg(feature = "uuid")]
impl ToDefault<D> for uuid::Uuid {
    fn to_default(self) -> String {
        format!("'{}'::uuid", self)
    }
}
#[cfg(feature = "uuid")]
impl ToDefault<D> for &uuid::Uuid {
    fn to_default(self) -> String {
        format!("'{}'::uuid", self)
    }
}

impl<T: ToDefault<D>> ToDefault<D> for Option<T> {
    fn to_default(self) -> String {
        match self {
            Some(v) => v.to_default(),
            None => "NULL".to_string(),
        }
    }
}
