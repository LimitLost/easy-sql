use easy_macros::always_context;

use crate::Driver;

/// Table metadata used by the query macros.
///
/// Prefer implementing this trait via the [`Table`](macro@crate::Table) derive macro or
/// [`table_join!`](crate::table_join); manual implementations may need updates across releases.
#[always_context]
pub trait Table<D: Driver>: Sized {
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    /// WARNING: This signature may change in future releases; prefer the macros above.
    ///
    /// The name and first argument are stable, but return type and more arguments can be added.
    fn table_joins(current_query: &mut String);
}
