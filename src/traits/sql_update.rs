use easy_macros::macros::always_context;

use crate::{Driver, SqlExpr};

#[always_context]
pub trait SqlUpdate<Table, D: Driver>: Sized {
    fn updates(&mut self) -> anyhow::Result<Vec<(String, SqlExpr<'_, D>)>>;
}
