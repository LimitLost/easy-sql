pub use anyhow::{Context, Error, Result};
pub use easy_macros::context;
pub use lazy_static::lazy_static;
pub use sqlx::{Arguments, Type, TypeInfo};

/// Used for compiler checks
///
/// Panics if called outside of `never_fn()` input block
///
pub fn never_any<T>() -> T {
    panic!("This function should never be called outside of never_fn() input block");
}
