//! Optional change-watcher: a transparent feed of committed row mutations for building sync layers.

/// The kind of row mutation observed.
///
/// Variants mirror the SQLite row-operation hook; an unrecognized opcode is dropped rather than represented.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeOp {
    /// A row was inserted.
    Insert,
    /// A row was updated.
    Update,
    /// A row was deleted.
    Delete,
}

/// One committed row mutation.
///
/// Fields:
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowChange {
    /// The SQL table name the row belongs to (as reported by the driver — includes internal tables like
    /// `sqlite_sequence`/`EasySqlTables`, so consumers should filter to the tables they sync).
    pub table: String,
    /// Whether the row was inserted, updated, or deleted.
    pub op: ChangeOp,
    /// The affected row's id — the SQLite `rowid`, which for an `INTEGER PRIMARY KEY [AUTOINCREMENT]` table is
    /// the primary key value. For UPDATE this is the post-image rowid; for DELETE the rowid that was removed.
    pub rowid: i64,
}

/// A sink for committed row mutations, registered once at database creation.
///
///
/// The SQLite commit hook runs synchronously on the driver's worker thread, so `on_commit` must be
/// cheap and non-blocking — the canonical implementation just hands `changes` to a channel that a background
/// task drains (reading the actual rows, building the outbound oplog, pushing them upstream). It is never given
/// the work of an uncommitted transaction: a rolled-back transaction's buffered changes are discarded.
pub trait ChangeWatcher: Send + Sync + std::fmt::Debug {
    /// Called once per committed transaction with every row mutation that transaction produced, in order.
    ///
    /// Batching by transaction lets the consumer apply a unit of work atomically downstream and keeps
    /// the per-row hook overhead off the critical path.
    fn on_commit(&self, changes: Vec<RowChange>);
}
