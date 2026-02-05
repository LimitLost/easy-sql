use anyhow::Context;
use easy_macros::{always_context, context};
use sqlx::Row;

use super::Postgres;
use crate::{EasyExecutor, traits::SetupSql};

#[derive(Debug)]
pub struct TableExists {
    pub name: &'static str,
}

#[always_context]
impl SetupSql<Postgres> for TableExists {
    type Output = bool;

    async fn query(self, exec: &mut impl EasyExecutor<Postgres>) -> anyhow::Result<Self::Output> {
        let query = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = 'public' AND table_name = '{}')",
            self.name
        );
        #[no_context]
        let result: bool = sqlx::query(&query)
            .fetch_one(exec.executor())
            .await
            .with_context(context!("table_name: {:?} | query: {:?}", self.name, query))?
            .get(0);
        Ok(result)
    }
}
