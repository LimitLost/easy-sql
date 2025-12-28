use easy_macros::always_context;

use crate::{Driver, DriverConnection};

#[always_context]
pub trait Table<D: Driver>: Sized
where
    DriverConnection<D>: Send + Sync,
{
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    /// WARNING: Signature of this function WILL change in future releases, use #[derive(Table)] and table_join! macros for automatic implementation
    ///
    /// Name and the first argument won't change, but return type and more arguments will be added in future
    fn table_joins(current_query: &mut String);
}
