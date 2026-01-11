mod clauses;
mod table_field;

pub use clauses::*;
pub use table_field::*;

use crate::Driver;

pub type SqlxQuery<'a, D> = sqlx::query::Query<
    'a,
    <D as Driver>::InternalDriver,
    <<D as Driver>::InternalDriver as sqlx::Database>::Arguments<'a>,
>;

pub type SqlxQueryBuilder<'a, D> =
    sqlx::query_builder::QueryBuilder<'a, <D as Driver>::InternalDriver>;
