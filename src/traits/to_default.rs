use crate::Driver;

/// Converts a Rust value into a SQL default expression.
///
/// Used by table/insert macros when emitting `DEFAULT` or default expressions.
pub trait ToDefault<D: Driver> {
    /// Returns a raw SQL string representing the default value for the type.
    fn to_default(self) -> String;
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
