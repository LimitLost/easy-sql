mod clauses;
mod column;
mod sql_value;
mod table_field;
pub use table_field::*;

pub use clauses::*;
pub use column::*;
use easy_macros::macros::always_context;
pub use sql_value::*;

mod sql;
pub use sql::*;
mod setup_sql;
pub use setup_sql::*;

use crate::Db;

type QueryTy<'a> = sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>;

pub struct QueryData<'a> {
    query: String,
    bindings: Vec<&'a SqlValueMaybeRef<'a>>,
}

#[always_context]
impl<'a> QueryData<'a> {
    pub fn sqlx(&'a self) -> QueryTy<'a> {
        let mut query = sqlx::query(&self.query);
        for binding in &self.bindings {
            query = query.bind(binding);
        }
        query
    }
}
