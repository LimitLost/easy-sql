use easy_macros::macros::always_context;

use crate::SqlValueMaybeRef;

#[always_context]
pub trait SqlUpdate<Table> {
    fn updates(&self) -> anyhow::Result<Vec<(String, SqlValueMaybeRef<'_>)>>;
}
