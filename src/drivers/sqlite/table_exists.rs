use std::ops::DerefMut;

use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::{Row, SqliteConnection};

use super::Sqlite;
use crate::SetupSql;

#[derive(Debug)]
pub struct TableExists {
    pub name: &'static str,
}

#[always_context]
impl SetupSql<Sqlite> for TableExists {
    type Output = bool;

    async fn query(
        self,
        exec: &mut (impl DerefMut<Target = SqliteConnection> + Send + Sync),
    ) -> anyhow::Result<Self::Output> {
        let query = format!(
            "SELECT EXISTS (SELECT * FROM sqlite_master WHERE type='table' AND name='{}')",
            self.name
        );
        #[no_context]
        let result: bool = sqlx::query(&query)
            .fetch_one(exec.deref_mut())
            .await
            .with_context(context!("table_name: {:?} | query: {:?}", self.name, query))?
            .get(0);
        Ok(result)
    }
}
