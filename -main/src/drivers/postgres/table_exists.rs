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
        // Scope the existence check to the connection's active schema via `current_schema()` (the head of
        // `search_path`) instead of a hardcoded `'public'`. Reason: schema-per-test isolation puts each test's
        // tables in its own schema, and other tests' schemas hold identically-named tables in the same database;
        // a `'public'`-only check would never see them (always false) and misfire migrations. In the default
        // per-database mode `current_schema()` resolves to `public`, so behavior is unchanged.
        let query = format!(
            "SELECT EXISTS (SELECT FROM information_schema.tables WHERE table_schema = current_schema() AND table_name = '{}')",
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
