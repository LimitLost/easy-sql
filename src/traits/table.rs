use easy_macros::always_context;

use crate::Driver;

#[always_context]
pub trait Table<D: Driver>: Sized {
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    /// WARNING: Signature of this function WILL change in future releases, use #[derive(Table)] and table_join! macros for automatic implementation
    ///
    /// Name and the first argument won't change, but return type and more arguments will be added in future
    fn table_joins(current_query: &mut String);
}

/// Marker trait for tables that are not created via table_join!.
#[diagnostic::on_unimplemented(message = "UPDATE and DELETE queries do not support joined tables.")]
pub trait NotJoinedTable {}
