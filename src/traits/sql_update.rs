use easy_macros::macros::always_context;

use crate::SqlExpr;

#[always_context]
pub trait SqlUpdate<Table> {
    fn updates(&mut self) -> anyhow::Result<Vec<(String, SqlExpr<'_>)>>;
}
