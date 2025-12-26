pub use anyhow::{Context, Error, Result};
pub use easy_macros::context;
pub use futures::FutureExt;
pub use futures_core::Stream;
pub use lazy_static::lazy_static;
pub use sqlx::{Arguments, Executor, QueryBuilder, Type, TypeInfo, query::Query};

/// Used for compiler checks, quickly creates a value of any type
///
/// Panics if called
///
pub fn never_any<T>() -> T {
    panic!(
        "This function should never be called, it's used to quickly create value for type in compiler checks"
    );
}
