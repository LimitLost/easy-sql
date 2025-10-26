mod clauses;
mod column;
mod table_field;
use std::fmt::Debug;

use anyhow::Context as _;
pub use table_field::*;

pub use clauses::*;
pub use column::*;
use easy_macros::macros::always_context;

mod sql;
pub use sql::*;

use crate::{Driver, DriverArguments};

pub type SqlxQuery<'a, D> = sqlx::query::Query<
    'a,
    <D as Driver>::InternalDriver,
    <<D as Driver>::InternalDriver as sqlx::Database>::Arguments<'a>,
>;

pub type SqlxQueryBuilder<'a, D> =
    sqlx::query_builder::QueryBuilder<'a, <D as Driver>::InternalDriver>;

use sqlx::{Arguments, IntoArguments};

pub struct QueryData<'a, D: Driver> {
    builder: SqlxQueryBuilder<'a, D>,
}

impl<'a, D: Driver> Debug for QueryData<'a, D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryData")
            .field("builder", &self.builder.sql())
            .finish()
    }
}

impl<'a, D: Driver> QueryData<'a, D> {
    pub fn new(query: String, bindings: DriverArguments<'a, D>) -> Self
    where
        DriverArguments<'a, D>: IntoArguments<'a, D::InternalDriver>,
    {
        Self {
            builder: SqlxQueryBuilder::<'a, D>::with_arguments(query, bindings),
        }
    }
}

#[always_context]
impl<'a, D: Driver> QueryData<'a, D> {
    /// SAFETY: The caller must ensure that the returned query does not outlive the QueryData.
    /// Explanation in src/database_structs/connection.rs  where this is used.
    pub(crate) unsafe fn sqlx(&mut self) -> SqlxQuery<'a, D> {
        unsafe { std::mem::transmute(self.builder.build()) }
    }
}

pub struct QueryBuilder<'a, D: Driver> {
    args: DriverArguments<'a, D>,
}

impl<'a, D: Driver> Default for QueryBuilder<'a, D> {
    fn default() -> Self {
        Self {
            args: DriverArguments::<'a, D>::default(),
        }
    }
}

impl<'a, D: Driver> std::fmt::Debug for QueryBuilder<'a, D>
where
    DriverArguments<'a, D>: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryBuilder")
            .field("args", &self.args)
            .finish()
    }
}

#[always_context]
impl<'a, D: Driver> QueryBuilder<'a, D> {
    /// Binds a value to the query.
    ///
    /// # Safety
    /// The caller must ensure that any borrowed data in `value` lives at least as long
    /// as the query execution. (Until the function call `Table::get`, `Table::update` etc.  ends)
    pub unsafe fn bind<'b, T>(&mut self, value: T) -> anyhow::Result<&mut Self>
    where
        T: 'b + sqlx::Encode<'b, D::InternalDriver> + sqlx::Type<D::InternalDriver>,
    {
        let args: &mut DriverArguments<'b, D> = unsafe { std::mem::transmute(&mut self.args) };
        args.add(
            #[context(no)]
            value,
        )
        .map_err(anyhow::Error::from_boxed)?;
        Ok(self)
    }

    pub(crate) fn args(self) -> DriverArguments<'a, D> {
        self.args
    }
}
