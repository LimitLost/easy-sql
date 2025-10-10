mod clauses;
mod column;
mod table_field;
pub use table_field::*;

pub use clauses::*;
pub use column::*;
use easy_macros::macros::always_context;

mod sql;
pub use sql::*;

use crate::Driver;

pub type SqlxQuery<'a, D: Driver> =
    sqlx::query::Query<'a, D::InternalDriver, <D::InternalDriver as sqlx::Database>::Arguments<'a>>;

#[derive(Debug)]
pub struct QueryData<'a, D: Driver> {
    query: String,
    bindings: Vec<&'a D::Value<'a>>,
}

#[always_context]
impl<'a, D: Driver> QueryData<'a, D> {
    pub fn sqlx(&'a self) -> SqlxQuery<'a, D> {
        let mut query = sqlx::query(&self.query);
        for binding in &self.bindings {
            query = query.bind(binding);
        }
        query
    }
}
