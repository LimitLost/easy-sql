#![allow(async_fn_in_trait)]
//! Database Setup shouldn't be split into multiple threads and we can't provide Send support for table creation

use easy_macros::always_context;

use crate::traits::{Driver, EasyExecutor};

#[always_context]
/// Defines part of the database to initialize
///
/// Prefer implementing this trait via the [`DatabaseSetup`](macro@crate::DatabaseSetup) derive macro;
/// manual implementations may need updates across releases.
pub trait DatabaseSetup<D: Driver + 'static> {
    /// Initializes this part of the database (its tables plus any pending migrations), idempotently.
    ///
    /// conn - pass a single [`Connection`](crate::Connection) or a transaction — **not** a raw connection
    /// pool. A migration step can issue several statements that must all run on the same connection (e.g. the
    /// SQLite add-foreign-key rebuild spans `PRAGMA foreign_keys` / `BEGIN` / … / `COMMIT`); a pool would
    /// scatter them across connections and silently break the rebuild.
    async fn setup(conn: &mut (impl EasyExecutor<D> + Send + Sync)) -> anyhow::Result<()>;
}
