use std::ops::Bound;
use std::{borrow::Cow, time::Duration};

use std::fmt::{Display, Write};

use anyhow::Context;
use easy_macros::always_context;
use pg_escape::quote_literal;
use sqlx::postgres::types::{Oid, PgCiText, PgInterval, PgLQuery, PgLTree};

use sqlx::postgres::types::{
    PgBox, PgCircle, PgCube, PgHstore, PgLSeg, PgLine, PgMoney, PgPath, PgPoint, PgPolygon, PgRange,
};

use crate::{ToDefault, impl_to_default_to_string_with_ref};

type D = super::Postgres;

macro_rules! impl_to_default_owned_and_ref {
    ($ty:ty, |$value:ident| $body:expr) => {
        impl ToDefault<D> for $ty {
            fn to_default(self) -> String {
                <&$ty as ToDefault<D>>::to_default(&self)
            }
        }

        impl ToDefault<D> for &$ty {
            fn to_default(self) -> String {
                let $value = self;
                $body
            }
        }
    };
}

#[cfg(feature = "ipnet")]
impl ToDefault<D> for sqlx::types::ipnet::IpNet {
    fn to_default(self) -> String {
        format!("'{}'::inet", self)
    }
}
#[cfg(feature = "ipnetwork")]
impl_to_default_owned_and_ref!(sqlx::types::ipnetwork::IpNetwork, |value| format!(
    "'{}'::inet",
    value
));

#[cfg(any(feature = "ipnet", feature = "ipnetwork"))]
impl_to_default_owned_and_ref!(std::net::IpAddr, |value| format!("'{}'::inet", value));

#[cfg(feature = "mac_address")]
impl_to_default_owned_and_ref!(sqlx::types::mac_address::MacAddress, |value| format!(
    "'{}'::macaddr",
    value
));

#[cfg(feature = "bit_vec")]
impl_to_default_owned_and_ref!(sqlx::types::BitVec, |value| bit_vec_to_postgres_varbit(
    value
));

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

#[cfg(feature = "bstr")]
impl_to_default_owned_and_ref!(sqlx::types::BString, |value| bytes_to_postgres_bytea(
    value.as_slice()
));

impl_to_default_owned_and_ref!(Duration, |value| format!(
    "'{} microseconds'::interval",
    value.as_micros()
));

impl_to_default_owned_and_ref!(PgInterval, |value| format!(
    "'{} months {} days {} microseconds'::interval",
    value.months, value.days, value.microseconds
));

impl_to_default_owned_and_ref!(Oid, |value| format!("{}::oid", value.0));

impl_to_default_owned_and_ref!(PgCiText, |value| format!(
    "{}::citext",
    quote_literal(&value.to_string())
));

impl_to_default_owned_and_ref!(PgLQuery, |value| format!(
    "{}::lquery",
    quote_literal(&value.to_string())
));

impl_to_default_owned_and_ref!(PgLTree, |value| format!(
    "{}::ltree",
    quote_literal(&value.to_string())
));

impl_to_default_owned_and_ref!(PgHstore, |value| {
    format!("{}::hstore", quote_literal(&hstore_to_postgres_text(value)))
});

impl_to_default_owned_and_ref!(PgMoney, |value| {
    format!("(({})::numeric / 100)::money", value.0)
});

#[cfg(any(feature = "time", feature = "chrono"))]
impl<Time, Offset> ToDefault<D> for sqlx::postgres::types::PgTimeTz<Time, Offset>
where
    Time: Display,
    Offset: Display,
{
    fn to_default(self) -> String {
        pg_timetz_to_default(&self)
    }
}

#[cfg(any(feature = "time", feature = "chrono"))]
impl<Time, Offset> ToDefault<D> for &sqlx::postgres::types::PgTimeTz<Time, Offset>
where
    Time: Display,
    Offset: Display,
{
    fn to_default(self) -> String {
        pg_timetz_to_default(self)
    }
}

impl_to_default_owned_and_ref!(PgCube, |value| format!(
    "{}::cube",
    quote_literal(&cube_to_postgres_text(value))
));

impl_to_default_owned_and_ref!(PgBox, |value| format!(
    "{}::box",
    quote_literal(&format!(
        "(({},{}) , ({},{}))",
        value.upper_right_x, value.upper_right_y, value.lower_left_x, value.lower_left_y
    ))
));

impl_to_default_owned_and_ref!(PgCircle, |value| format!(
    "{}::circle",
    quote_literal(&format!("<({},{}),{}>", value.x, value.y, value.radius))
));

impl_to_default_owned_and_ref!(PgLSeg, |value| format!(
    "{}::lseg",
    quote_literal(&format!(
        "(({},{}) , ({},{}))",
        value.start_x, value.start_y, value.end_x, value.end_y
    ))
));

impl_to_default_owned_and_ref!(PgLine, |value| format!(
    "{}::line",
    quote_literal(&format!("{{{},{},{}}}", value.a, value.b, value.c))
));

impl_to_default_owned_and_ref!(PgPath, |value| format!(
    "{}::path",
    quote_literal(&path_to_postgres_text(value))
));

impl_to_default_owned_and_ref!(PgPoint, |value| format!(
    "{}::point",
    quote_literal(&format!("({},{})", value.x, value.y))
));

impl_to_default_owned_and_ref!(PgPolygon, |value| format!(
    "{}::polygon",
    quote_literal(&polygon_to_postgres_text(value))
));

impl<T> ToDefault<D> for PgRange<T>
where
    T: Display,
{
    fn to_default(self) -> String {
        <&PgRange<T> as ToDefault<D>>::to_default(&self)
    }
}

impl<T> ToDefault<D> for &PgRange<T>
where
    T: Display,
{
    fn to_default(self) -> String {
        let lower_bracket = match self.start {
            Bound::Included(_) => '[',
            _ => '(',
        };
        let upper_bracket = match self.end {
            Bound::Included(_) => ']',
            _ => ')',
        };

        let start = format_pg_range_bound(&self.start);
        let end = format_pg_range_bound(&self.end);

        let text = format!("{lower_bracket}{start},{end}{upper_bracket}");
        quote_literal(&text)
    }
}

#[always_context(skip(!))]
impl<T> ToDefault<D> for sqlx::types::Json<T>
where
    T: serde::Serialize,
{
    fn to_default_failable(self) -> anyhow::Result<String> {
        <&sqlx::types::Json<T> as ToDefault<D>>::to_default_failable(&self)
    }
}

#[always_context(skip(!))]
impl<T> ToDefault<D> for &sqlx::types::Json<T>
where
    T: serde::Serialize,
{
    fn to_default_failable(self) -> anyhow::Result<String> {
        let json = self.encode_to_string()?;
        Ok(format!("{}::jsonb", quote_literal(&json)))
    }
}

impl<T> ToDefault<D> for sqlx::types::Text<T>
where
    T: Display,
{
    fn to_default(self) -> String {
        quote_literal(&self.0.to_string())
    }
}

impl<T> ToDefault<D> for &sqlx::types::Text<T>
where
    T: Display,
{
    fn to_default(self) -> String {
        quote_literal(&self.0.to_string())
    }
}

#[cfg(feature = "json")]
impl_to_default_owned_and_ref!(sqlx::types::JsonValue, |value| format!(
    "{}::jsonb",
    quote_literal(&value.to_string())
));

#[cfg(feature = "time")]
impl_to_default_owned_and_ref!(sqlx::types::time::Date, |value| format!(
    "{}::date",
    quote_literal(&value.to_string())
));

#[cfg(feature = "time")]
impl_to_default_owned_and_ref!(sqlx::types::time::Time, |value| format!(
    "{}::time",
    quote_literal(&value.to_string())
));

#[cfg(feature = "time")]
impl_to_default_owned_and_ref!(sqlx::types::time::PrimitiveDateTime, |value| format!(
    "{}::timestamp",
    quote_literal(&value.to_string())
));

#[cfg(feature = "time")]
impl_to_default_owned_and_ref!(sqlx::types::time::OffsetDateTime, |value| format!(
    "{}::timestamptz",
    quote_literal(&value.to_string())
));

#[cfg(feature = "bigdecimal")]
impl_to_default_owned_and_ref!(bigdecimal::BigDecimal, |value| format!("{value}::numeric",));
#[cfg(feature = "rust_decimal")]
impl_to_default_owned_and_ref!(rust_decimal::Decimal, |value| format!("{value}::numeric",));

#[cfg(feature = "chrono")]
impl_to_default_owned_and_ref!(chrono::TimeDelta, |value| format!(
    "'{} seconds {} microseconds'::interval",
    value.num_seconds(),
    value.subsec_nanos() / 1000
));

impl_to_default_owned_and_ref!(Vec<u8>, |value| bytes_to_postgres_bytea(value));

impl ToDefault<D> for &[u8] {
    fn to_default(self) -> String {
        bytes_to_postgres_bytea(self)
    }
}

#[cfg(feature = "chrono")]
impl_to_default_owned_and_ref!(chrono::NaiveDate, |value| format!(
    "'{}'::date",
    value.format("%Y-%m-%d")
));

#[cfg(feature = "chrono")]
impl_to_default_owned_and_ref!(chrono::NaiveDateTime, |value| format!(
    "'{}'::timestamp",
    value.format("%Y-%m-%d %H:%M:%S%.f")
));

#[cfg(feature = "chrono")]
impl_to_default_owned_and_ref!(chrono::NaiveTime, |value| format!(
    "'{}'::time",
    value.format("%H:%M:%S%.f")
));
#[cfg(feature = "uuid")]
impl_to_default_owned_and_ref!(uuid::Uuid, |value| format!("'{}'::uuid", value));

#[always_context(skip(!))]
impl<T: ToDefault<D>> ToDefault<D> for Option<T> {
    fn to_default(self) -> String {
        match self {
            Some(v) => v.to_default(),
            None => "NULL".to_string(),
        }
    }

    fn to_default_failable(self) -> anyhow::Result<String> {
        match self {
            Some(v) => v.to_default_failable(),
            None => Ok("NULL".to_string()),
        }
    }
}

#[cfg(feature = "bit_vec")]
fn bit_vec_to_postgres_varbit(bits: &sqlx::types::BitVec) -> String {
    let mut output = String::with_capacity(bits.len() + 11);
    output.push_str("B'");

    for bit in bits.iter() {
        output.push(if bit { '1' } else { '0' });
    }

    output.push_str("'::varbit");
    output
}

fn bytes_to_postgres_bytea(bytes: &[u8]) -> String {
    const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

    let mut output = String::with_capacity(bytes.len() * 2 + 11);
    output.push_str("'\\x");

    for &byte in bytes {
        output.push(HEX_LOWER[(byte >> 4) as usize] as char);
        output.push(HEX_LOWER[(byte & 0x0f) as usize] as char);
    }

    output.push_str("'::bytea");
    output
}

fn escape_hstore_string(value: &str) -> String {
    escape_backslashes_and_double_quotes(value)
}

fn hstore_to_postgres_text(value: &PgHstore) -> String {
    let mut output = String::new();

    for (index, (key, val)) in value.0.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        let escaped_key = escape_hstore_string(key);
        output.push('"');
        output.push_str(&escaped_key);
        output.push_str("\"=>");

        match val {
            Some(v) => {
                let escaped_v = escape_hstore_string(v);
                output.push('"');
                output.push_str(&escaped_v);
                output.push('"');
            }
            None => output.push_str("NULL"),
        }
    }

    output
}

fn cube_to_postgres_text(value: &PgCube) -> String {
    match value {
        PgCube::Point(v) => format!("({v})"),
        PgCube::ZeroVolume(values) => {
            let mut output = String::from("(");
            for (index, point) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                let _ = write!(output, "{point}");
            }
            output.push(')');
            output
        }
        PgCube::OneDimensionInterval(start, end) => format!("({start}),({end})"),
        PgCube::MultiDimension(points) => {
            let mut output = String::new();
            for (point_index, point) in points.iter().enumerate() {
                if point_index > 0 {
                    output.push(',');
                }

                output.push('(');
                for (value_index, value) in point.iter().enumerate() {
                    if value_index > 0 {
                        output.push(',');
                    }
                    let _ = write!(output, "{value}");
                }
                output.push(')');
            }
            output
        }
    }
}

fn path_to_postgres_text(value: &PgPath) -> String {
    let mut output = String::new();
    output.push(if value.closed { '(' } else { '[' });

    for (index, point) in value.points.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        output.push('(');
        let _ = write!(output, "{}", point.x);
        output.push(',');
        let _ = write!(output, "{}", point.y);
        output.push(')');
    }

    output.push(if value.closed { ')' } else { ']' });
    output
}

fn polygon_to_postgres_text(value: &PgPolygon) -> String {
    let mut output = String::from("(");
    for (index, point) in value.points.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }

        output.push('(');
        let _ = write!(output, "{}", point.x);
        output.push(',');
        let _ = write!(output, "{}", point.y);
        output.push(')');
    }
    output.push(')');
    output
}

fn escape_pg_range_value(value: &str) -> String {
    escape_backslashes_and_double_quotes(value)
}

fn escape_backslashes_and_double_quotes(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 4);
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn format_pg_range_bound<T: Display>(bound: &Bound<T>) -> String {
    match bound {
        Bound::Included(v) | Bound::Excluded(v) => {
            let rendered = escape_pg_range_value(&v.to_string());
            format!("\"{rendered}\"")
        }
        Bound::Unbounded => String::new(),
    }
}

#[cfg(any(feature = "time", feature = "chrono"))]
fn pg_timetz_to_default<Time, Offset>(
    value: &sqlx::postgres::types::PgTimeTz<Time, Offset>,
) -> String
where
    Time: Display,
    Offset: Display,
{
    let text = format!("{}{}", value.time, value.offset);
    format!("{}::timetz", quote_literal(&text))
}
