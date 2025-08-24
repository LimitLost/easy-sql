use easy_macros::macros::always_context;

use crate::{SqlExpr, SqlUpdate};

#[derive(Debug)]
pub struct UpdateSetClause<'a> {
    pub updates: Vec<(String, SqlExpr<'a>)>,
}

#[always_context]
impl<T> SqlUpdate<T> for UpdateSetClause<'_> {
    fn updates(&mut self) -> anyhow::Result<Vec<(String, SqlExpr<'_>)>> {
        let all_data = self.updates.drain(..).collect();

        Ok(all_data)
    }
}
