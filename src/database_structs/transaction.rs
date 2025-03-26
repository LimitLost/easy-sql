use std::sync::Arc;

use easy_macros::macros::always_context;
use tokio::sync::Mutex;

use super::DatabaseInternal;
use crate::Db;

pub struct Transaction {
    conn: sqlx::Transaction<'static, Db>,
    db_link: Arc<Mutex<DatabaseInternal>>,
}

#[always_context]
impl Transaction {
    pub(crate) fn new(
        conn: sqlx::Transaction<'static, Db>,
        db_link: Arc<Mutex<DatabaseInternal>>,
    ) -> Self {
        Transaction { conn, db_link }
    }
}
