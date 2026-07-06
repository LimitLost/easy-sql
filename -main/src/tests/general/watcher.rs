//! Tests for the opt-in change-watcher (`watcher` feature): committed row mutations are reported via sqlite's
//! native hooks, rolled-back ones are not, and the reported `(table, op, rowid)` matches the work done.

use super::Database;
use crate::watcher::{ChangeOp, ChangeWatcher, RowChange};
use crate::{Insert, Table, Update};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;
use std::sync::{Arc, Mutex};

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
#[sql(table_name = "watcher_test")]
struct WatcherTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    value: String,
}

#[derive(Insert, Debug, Clone)]
#[sql(table = WatcherTestTable)]
#[sql(default = id)]
struct WatcherTestInsert {
    value: String,
}

#[derive(Update, Debug, Clone)]
#[sql(table = WatcherTestTable)]
struct WatcherTestUpdate {
    value: String,
}

/// A watcher that accumulates every reported change into a shared vec, for assertions.
#[derive(Debug)]
struct CollectingWatcher {
    events: Arc<Mutex<Vec<RowChange>>>,
}

impl ChangeWatcher for CollectingWatcher {
    fn on_commit(&self, changes: Vec<RowChange>) {
        self.events.lock().unwrap().extend(changes);
    }
}

/// Builds a watcher-armed temp database for the test (mirrors the temp-path pattern used by the pool tests).
#[always_context(skip(!))]
async fn setup_watched_db(watcher: Arc<dyn ChangeWatcher>) -> anyhow::Result<Database> {
    use crate::tests::init_test_logger;
    use sqlx::sqlite::SqliteConnectOptions;

    init_test_logger();

    let base_dir = std::env::temp_dir().join("easy_sql_watcher_tests");
    std::fs::create_dir_all(&base_dir)?;
    let now_nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .context("System time is before UNIX_EPOCH")?
        .as_nanos();
    let path = base_dir.join(format!("watcher_test_{now_nanos}.sqlite"));

    Database::setup_with_watcher::<WatcherTestTable>(
        SqliteConnectOptions::default()
            .filename(&path)
            .create_if_missing(true),
        watcher,
    )
    .await
}

/// The watcher reports INSERT/UPDATE/DELETE for committed transactions in order, with the affected rowid, and
/// reports nothing for a rolled-back transaction.
#[always_context(skip(!))]
#[tokio::test]
async fn watcher_reports_committed_changes_only() -> anyhow::Result<()> {
    let events: Arc<Mutex<Vec<RowChange>>> = Arc::new(Mutex::new(Vec::new()));
    let db = setup_watched_db(Arc::new(CollectingWatcher {
        events: events.clone(),
    }))
    .await?;

    // Only assert on our own table — the watcher also reports internal writes (EasySqlTables, sqlite_sequence).
    let collected = || -> Vec<(ChangeOp, i64)> {
        events
            .lock()
            .unwrap()
            .iter()
            .filter(|change| change.table == "watcher_test")
            .map(|change| (change.op, change.rowid))
            .collect()
    };

    // - INSERT in a committed transaction → one Insert at rowid 1.
    {
        let mut tx = db.transaction().await?;
        let row = WatcherTestInsert {
            value: "a".to_string(),
        };
        query!(&mut tx, INSERT INTO WatcherTestTable VALUES {row}).await?;
        tx.commit().await?;
    }
    assert_eq!(collected(), vec![(ChangeOp::Insert, 1)]);

    // - UPDATE in a committed transaction → an Update at rowid 1.
    {
        let mut tx = db.transaction().await?;
        let update = WatcherTestUpdate {
            value: "b".to_string(),
        };
        query!(&mut tx, UPDATE WatcherTestTable SET {update} WHERE id = 1).await?;
        tx.commit().await?;
    }
    assert_eq!(collected(), vec![(ChangeOp::Insert, 1), (ChangeOp::Update, 1)]);

    // - INSERT then ROLLBACK → no new event reported.
    {
        let mut tx = db.transaction().await?;
        let row = WatcherTestInsert {
            value: "c".to_string(),
        };
        query!(&mut tx, INSERT INTO WatcherTestTable VALUES {row}).await?;
        tx.rollback().await?;
    }
    assert_eq!(collected(), vec![(ChangeOp::Insert, 1), (ChangeOp::Update, 1)]);

    // - DELETE in a committed transaction → a Delete at rowid 1.
    {
        let mut tx = db.transaction().await?;
        query!(&mut tx, DELETE FROM WatcherTestTable WHERE id = 1).await?;
        tx.commit().await?;
    }
    assert_eq!(
        collected(),
        vec![
            (ChangeOp::Insert, 1),
            (ChangeOp::Update, 1),
            (ChangeOp::Delete, 1)
        ]
    );

    // - Autocommit path (the dominant app pattern: a single `query!` on a plain connection, no explicit
    //   transaction) must also fire the commit hook → an Insert at the next rowid (2).
    {
        let mut conn = db.conn().await?;
        let row = WatcherTestInsert {
            value: "d".to_string(),
        };
        query!(&mut conn, INSERT INTO WatcherTestTable VALUES {row}).await?;
    }
    assert_eq!(
        collected(),
        vec![
            (ChangeOp::Insert, 1),
            (ChangeOp::Update, 1),
            (ChangeOp::Delete, 1),
            (ChangeOp::Insert, 2)
        ]
    );

    Ok(())
}
