use easy_macros::always_context;

use crate::Driver;

#[always_context(skip(!))]
/// Converts a Rust value into a SQL default expression.
///
/// Used by table/insert macros when emitting `DEFAULT` or default expressions.
///
/// Implement at least one of these methods:
/// - Implement `to_default` for infallible conversions.
/// - Implement `to_default_failable` when conversion can fail.
///
/// The default `to_default` implementation fails by default
pub trait ToDefault<D: Driver> {
    /// Returns a raw SQL string representing the default value for the type.
    fn to_default(self) -> String
    where
        Self: std::marker::Sized,
    {
        panic!(
            "ToDefault::to_default is not implemented for this type. Implement either `to_default` or `to_default_failable`."
        )
    }

    /// Returns a raw SQL string representing the default value for the type.
    ///
    /// By default this delegates to `to_default`.
    fn to_default_failable(self) -> anyhow::Result<String>
    where
        Self: std::marker::Sized,
    {
        Ok(self.to_default())
    }
}
#[macro_export]
#[doc(hidden)]
/// Support auto implement because I'm lazy
macro_rules! impl_to_default_to_string_with_ref {
    ($t:ty) => {
        impl ToDefault<D> for $t {
            fn to_default(self) -> String {
                self.to_string()
            }
        }

        impl<'a> ToDefault<D> for &'a $t {
            fn to_default(self) -> String {
                self.to_string()
            }
        }
    };
}
