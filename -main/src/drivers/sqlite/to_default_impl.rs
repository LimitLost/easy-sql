use crate::{ToDefault, impl_to_default_to_string_with_ref};

type D = super::Sqlite;

impl_to_default_to_string_with_ref!(bool);
impl_to_default_to_string_with_ref!(f32);
impl_to_default_to_string_with_ref!(f64);
impl_to_default_to_string_with_ref!(i8);
impl_to_default_to_string_with_ref!(i16);
impl_to_default_to_string_with_ref!(i32);
impl_to_default_to_string_with_ref!(i64);

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

impl ToDefault<D> for String {
    fn to_default(self) -> String {
        format!("'{}'", escape_sql(&self))
    }
}

impl ToDefault<D> for &String {
    fn to_default(self) -> String {
        format!("'{}'", escape_sql(self))
    }
}

impl ToDefault<D> for &str {
    fn to_default(self) -> String {
        format!("'{}'", escape_sql(self))
    }
}
#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::NaiveDate {
    fn to_default(self) -> String {
        format!("'{}'", self.format("%F"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::NaiveDate {
    fn to_default(self) -> String {
        format!("'{}'", self.format("%F"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::NaiveDateTime {
    fn to_default(self) -> String {
        format!("'{}'", self.format("%F %T%.f"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::NaiveDateTime {
    fn to_default(self) -> String {
        format!("'{}'", self.format("%F %T%.f"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for chrono::NaiveTime {
    fn to_default(self) -> String {
        format!("'{}'", self.format("%T%.f"))
    }
}

#[cfg(feature = "chrono")]
impl ToDefault<D> for &chrono::NaiveTime {
    fn to_default(self) -> String {
        format!("'{}'", self.format("%T%.f"))
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
