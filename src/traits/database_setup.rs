use easy_macros::always_context;

use crate::traits::{Driver, EasyExecutor};

#[always_context]
/// Defines part of the database to initialize
///
/// Prefer implementing this trait via the [`DatabaseSetup`](crate::DatabaseSetup) derive macro;
/// manual implementations may need updates across releases.
pub trait DatabaseSetup<D: Driver + 'static> {
    async fn setup(conn: &mut (impl EasyExecutor<D> + Send + Sync)) -> anyhow::Result<()>;
}
