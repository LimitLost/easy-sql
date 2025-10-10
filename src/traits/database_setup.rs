use easy_macros::macros::always_context;

use crate::{Driver, easy_executor::EasyExecutor};

#[always_context]
pub trait DatabaseSetup<D: Driver + 'static> {
    async fn setup(conn: &mut (impl EasyExecutor<D> + Send + Sync)) -> anyhow::Result<()>;
}
