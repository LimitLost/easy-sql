use easy_macros::macros::always_context;

use crate::{Driver, SqlExpr, SqlUpdate};

#[derive(Debug)]
pub struct UpdateSetClause<'a, D: Driver> {
    pub updates: Vec<(String, SqlExpr<'a, D>)>,
}

#[always_context]
impl<Table, D: Driver> SqlUpdate<Table, D> for UpdateSetClause<'_, D> {
    fn updates(&mut self) -> anyhow::Result<Vec<(String, SqlExpr<'_, D>)>> {
        let all_data: Vec<(String, SqlExpr<'_, D>)> = self
            .updates
            .drain(..)
            .map(|(name, expr)| {
                // SAFETY: SqlExpr should be covariant over its lifetime parameter, but due to
                // the presence of D::Value<'a> , the lifetime becomes invariant.
                // This wasn't a problem until Driver with ability to select custom Value was added.
                // See https://github.com/LimitLost/easy-sql/blob/f28dd681c890a58fe0ae3526c9bfd934aab8a79e/src/sql_query/clauses/update_set_clause.rs
                (name, unsafe {
                    std::mem::transmute::<SqlExpr<'_, D>, SqlExpr<'_, D>>(expr)
                })
            })
            .collect();

        Ok(all_data)
    }
}
