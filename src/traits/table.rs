use easy_macros::always_context;

use crate::QueryBuilder;
use crate::{Driver, DriverConnection, TableJoin};

#[always_context]
pub trait Table<D: Driver>: Sized
where
    DriverConnection<D>: Send + Sync,
{
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    fn table_joins(builder: &mut QueryBuilder<'_, D>) -> Vec<TableJoin>;
}
