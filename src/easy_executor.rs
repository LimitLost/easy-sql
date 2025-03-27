use async_trait::async_trait;
use easy_macros::macros::always_context;
use futures::{TryStream, stream::BoxStream};

use crate::{Row, SqlOutput, ToConvert, sql_query::Sql};

pub struct Break;

#[always_context]
#[async_trait]
pub trait EasyExecutor {
    // async fn query(&mut self, sql: &Sql) -> anyhow::Result<()>;
    async fn query<Y: ToConvert + Send + Sync, T, O: SqlOutput<T, Y>>(
        &mut self,
        sql: &Sql,
    ) -> anyhow::Result<O>;
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
