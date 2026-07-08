use anyhow::Context;
use easy_macros::always_context;

use std::path::Path;
#[cfg(test)]
use std::path::PathBuf;

use crate::{Connection, DatabaseSetup, EasySqlTables, PoolTransaction};

use super::Db;

pub use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};

use super::Sqlite;

/// SQLite connection pool wrapper with setup helpers.
///
/// Uses [`DatabaseSetup`](crate::DatabaseSetup) implementations to prepare schema on startup.
#[derive(Debug)]
pub struct Database {
    connection_pool: sqlx::Pool<Db>,
    #[cfg(test)]
    pub test_db_file_path: Option<PathBuf>,
}

#[cfg(test)]
impl Drop for Database {
    fn drop(&mut self) {
        if let Some(path) = &self.test_db_file_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[always_context]
impl Database {
    pub async fn setup<T: DatabaseSetup<Sqlite>>(
        db_file_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&db_file_path)
                .create_if_missing(true),
        )
        .await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: Some(db_file_path.as_ref().to_path_buf()),
        })
    }

    pub async fn setup_with_options<T: DatabaseSetup<Sqlite>>(
        options: SqliteConnectOptions,
    ) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect_with(options.clone()).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: Some(options.get_filename().to_owned()),
        })
    }

    /// Like [`setup_with_options`](Self::setup_with_options), but arms a [`ChangeWatcher`](crate::ChangeWatcher)
    /// on every pooled connection so committed row mutations are reported without changing any `query!` call site.
    ///
    /// Limitations (inherent to sqlite's update/commit/rollback hooks):
    /// - `SAVEPOINT` rollbacks are invisible: the rollback hook fires only for the outermost transaction, so a row
    ///   change made inside a `ROLLBACK TO SAVEPOINT`-undone nested block is still reported at the outer commit
    ///   (over-report). A consumer that reads the row fresh sees its reverted state, so read-through sync self-corrects.
    /// - `on_commit` runs synchronously on sqlite's worker thread inside the commit hook: it must not panic (a panic
    ///   unwinds across the C hook boundary) and must be cheap (it blocks the committing transaction).
    /// - `WITHOUT ROWID` tables are never observed — sqlite's update hook does not fire for them.
    #[cfg(feature = "watcher")]
    #[no_context_inputs]
    pub async fn setup_with_watcher<T: DatabaseSetup<Sqlite>>(
        options: SqliteConnectOptions,
        watcher: std::sync::Arc<dyn crate::watcher::ChangeWatcher>,
    ) -> anyhow::Result<Self> {
        // build the pool with an after_connect hook so EVERY connection arms the watcher — the hooks are
        // per-connection sqlite state and the pool may open many connections over its lifetime.
        let connection_pool = sqlx::sqlite::SqlitePoolOptions::new()
            .after_connect(move |conn, _meta| {
                let watcher = watcher.clone();
                Box::pin(async move { install_watcher_hooks(conn, watcher).await })
            })
            .connect_with(options.clone())
            .await?;

        // identical schema bring-up to setup_with_options.
        let mut conn = Connection::new(connection_pool.acquire().await?);
        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: Some(options.get_filename().to_owned()),
        })
    }

    // Broken - database will be lost after connection is closed
    /* pub async fn setup_in_memory<T: DatabaseSetup<Sqlite>>() -> anyhow::Result<Self> {
        let connection_pool =
            sqlx::Pool::<Db>::connect_with(SqliteConnectOptions::default().in_memory(true)).await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            #[cfg(test)]
            test_db_file_path: None,
        })
    } */

    pub async fn conn(&self) -> anyhow::Result<Connection<Sqlite>> {
        let conn = self.connection_pool.acquire().await?;
        Ok(Connection::new(conn))
    }

    pub async fn transaction(&self) -> anyhow::Result<PoolTransaction<Sqlite>> {
        let conn = self.connection_pool.begin().await?;
        Ok(PoolTransaction::new(conn))
    }

    /// Returns the underlying connection pool — needed by sync layers that arm sqlite hooks on connections
    /// they open themselves, or that run raw `sqlx` against sync-owned bookkeeping tables to avoid recursion.
    ///
    /// The watcher reports changes but a sync engine also needs direct pool access for its side tables.
    #[cfg(feature = "watcher")]
    pub fn pool(&self) -> &sqlx::Pool<Db> {
        &self.connection_pool
    }
    #[cfg(test)]
    pub async fn setup_for_testing<T: DatabaseSetup<Sqlite>>() -> anyhow::Result<Self> {
        use tokio::sync::Mutex;

        use crate::tests::init_test_logger;

        init_test_logger();

        lazy_static::lazy_static! {
            static ref CURRENT_NAME_N:Mutex<usize>=Default::default();
        }
        let current_path = std::env::current_dir()?;
        let test_db_path = {
            let mut current_n = CURRENT_NAME_N.lock().await;
            let path = current_path.join(format!("test_db_{}", *current_n));
            *current_n += 1;
            path
        };

        let connection_pool = sqlx::Pool::<Db>::connect_with(
            SqliteConnectOptions::default()
                .filename(&test_db_path)
                .create_if_missing(true),
        )
        .await?;

        let mut conn = Connection::new(connection_pool.acquire().await?);

        EasySqlTables::setup(&mut &mut conn).await?;
        T::setup(&mut &mut conn).await?;

        Ok(Database {
            connection_pool,
            test_db_file_path: Some(test_db_path),
        })
    }
}

/// Arms sqlite's update/commit/rollback hooks on one connection, buffering row changes until commit.
///
/// The update hook fires per row *during* a transaction, so the changes are only real once it commits —
/// the commit hook flushes the buffer to the watcher (and lets the commit proceed), the rollback hook discards
/// it. Installed per connection by `setup_with_watcher`'s `after_connect`.
#[cfg(feature = "watcher")]
async fn install_watcher_hooks(
    conn: &mut sqlx::sqlite::SqliteConnection,
    watcher: std::sync::Arc<dyn crate::watcher::ChangeWatcher>,
) -> Result<(), sqlx::Error> {
    use crate::watcher::{ChangeOp, RowChange};
    use sqlx::sqlite::{SqliteOperation, UpdateHookResult};
    use std::sync::{Arc, Mutex};

    // Per-connection buffer shared by the three hooks. They run serially on this connection's worker thread, so
    // the mutex is effectively uncontended — it is here only to satisfy the hooks' `Send + 'static` bounds.
    let buffer: Arc<Mutex<Vec<RowChange>>> = Arc::new(Mutex::new(Vec::new()));
    let mut handle = conn.lock_handle().await?;

    // - update hook: record each affected row (table + op + rowid) into the pending buffer.
    let update_buffer = buffer.clone();
    handle.set_update_hook(move |result: UpdateHookResult| {
        let op = match result.operation {
            SqliteOperation::Insert => ChangeOp::Insert,
            SqliteOperation::Update => ChangeOp::Update,
            SqliteOperation::Delete => ChangeOp::Delete,
            SqliteOperation::Unknown(_) => return,
        };
        if let Ok(mut pending) = update_buffer.lock() {
            pending.push(RowChange {
                table: result.table.to_string(),
                op,
                rowid: result.rowid,
            });
        }
    });

    // - commit hook: the transaction is committing — drain the buffer to the watcher, then allow the commit by
    //   returning true (sqlx turns a `false` return into a ROLLBACK).
    let commit_buffer = buffer.clone();
    let commit_watcher = watcher.clone();
    handle.set_commit_hook(move || {
        let drained = commit_buffer
            .lock()
            .map(|mut pending| std::mem::take(&mut *pending))
            .unwrap_or_default();
        if !drained.is_empty() {
            commit_watcher.on_commit(drained);
        }
        true
    });

    // - rollback hook: the transaction was abandoned — discard its buffered changes.
    let rollback_buffer = buffer.clone();
    handle.set_rollback_hook(move || {
        if let Ok(mut pending) = rollback_buffer.lock() {
            pending.clear();
        }
    });

    Ok(())
}
