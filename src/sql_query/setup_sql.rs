use anyhow::Context;
use async_trait::async_trait;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::Row;

use crate::SetupSql;

pub struct TableExists {
    name: &'static str,
}

#[always_context]
#[async_trait]
impl SetupSql for TableExists {
    type Output = bool;

    async fn query<'a>(
        self,
        exec: impl sqlx::Executor<'a, Database = crate::Db> + Send + Sync,
    ) -> anyhow::Result<Self::Output> {
        let query = format!(
            "SELECT EXISTS (SELECT * FROM sqlite_master WHERE type='table' AND name='{}')",
            self.name
        );
        #[no_context]
        let result: bool = sqlx::query(&query)
            .fetch_one(exec)
            .await
            .with_context(context!("table_name: {:?} | query: {:?}", self.name, query))?
            .get(0);
        Ok(result)
    }
}
