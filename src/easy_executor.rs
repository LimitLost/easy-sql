use async_trait::async_trait;
use easy_macros::macros::always_context;

use crate::{Row, SqlOutput, sql_query::Sql};

#[always_context]
#[async_trait]
pub trait EasyExecutor {
    async fn query(&mut self, sql: &Sql) -> anyhow::Result<()>;
    async fn fetch_one<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<O>;
    async fn fetch_many<T, O: SqlOutput<T, Row>>(&mut self, sql: &Sql) -> anyhow::Result<Vec<O>>;
}
