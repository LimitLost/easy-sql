use async_trait::async_trait;
use easy_macros::macros::always_context;

use crate::easy_executor::EasyExecutor;

#[always_context]
#[async_trait]
pub trait DatabaseSetup {
    async fn setup(
        conn: &mut (impl EasyExecutor + Send + Sync),
        used_table_names: &mut Vec<String>,
    ) -> anyhow::Result<()>;
}
