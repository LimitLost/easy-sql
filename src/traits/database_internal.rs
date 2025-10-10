use std::fmt::Debug;

use easy_macros::macros::always_context;

use crate::{Driver, Sql};

#[always_context]
pub trait DatabaseInternal: Debug {
    type Driver: Driver;

    async fn sql_request(&mut self, sql: &Sql<'_, Self::Driver>) -> anyhow::Result<()>;
    async fn maybe_exit(&mut self) -> anyhow::Result<()>;
}
