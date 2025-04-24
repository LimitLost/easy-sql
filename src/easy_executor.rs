use std::{fmt::Debug, ops::DerefMut};

use async_trait::async_trait;
use easy_macros::macros::always_context;

use crate::{RawConnection, Row, SqlOutput, ToConvert, sql_query::Sql};

pub struct Break;

#[always_context]
#[async_trait]
pub trait EasyExecutor: Debug {
    // async fn query(&mut self, sql: &Sql) -> anyhow::Result<()>;
    async fn query<Y: ToConvert + Send + Sync, T, O: SqlOutput<T, Y>>(
        &mut self,
        sql: &Sql,
    ) -> anyhow::Result<O>;

    async fn query_setup<O: SetupSql + Send + Sync>(&mut self, sql: O)
    -> anyhow::Result<O::Output>;

    // async fn fetch_all<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<Vec<O>>;

    ///# How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn fetch_lazy<T, O: SqlOutput<T, Row>>(
        &mut self,
        sql: &Sql,
        mut perform: impl FnMut(O) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>;
}

#[always_context]
#[async_trait]
pub trait SetupSql {
    type Output;

    async fn query<'a>(
        self,
        exec: &mut (impl DerefMut<Target = RawConnection> + Send + Sync),
    ) -> anyhow::Result<Self::Output>;
}
