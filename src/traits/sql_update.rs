use easy_macros::macros::always_context;

use crate::SqlValueMaybeRef;

#[always_context]
pub trait SqlUpdate<Table> {
    fn updates(&self) -> Vec<(String, SqlValueMaybeRef<'_>)>;
}
