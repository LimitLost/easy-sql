use crate::Driver;

pub trait ToDefault<D: Driver> {
    /// Returns a raw SQL string representing the default value for the type.
    fn to_default(self) -> String;
}
#[macro_export]
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
