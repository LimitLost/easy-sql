use std::ops::DerefMut;

use anyhow::Context;
use async_trait::async_trait;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::Row;

use crate::{RawConnection, SetupSql};

#[derive(Debug)]
pub struct TableExists {
    pub name: &'static str,
}

#[always_context]
#[async_trait]
impl SetupSql for TableExists {
    type Output = bool;

    async fn query<'a>(
        self,
        exec: &mut (impl DerefMut<Target = RawConnection> + Send + Sync),
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
