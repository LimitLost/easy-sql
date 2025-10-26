#[cfg(test)]
use std::path::PathBuf;

use easy_macros::macros::always_context;

use crate::{DatabaseInternal, Sql};

/// TODO Will be used in the future to send data to the remote database
#[derive(Debug, Default)]
pub struct DatabaseInternalDefault {
    #[cfg(test)]
    pub test_db_file_path: Option<PathBuf>,
}
#[cfg(test)]
impl Drop for DatabaseInternalDefault {
    fn drop(&mut self) {
        if let Some(path) = &self.test_db_file_path {
            let _ = std::fs::remove_file(path);
        }
    }
}

#[always_context]
impl DatabaseInternal for DatabaseInternalDefault {
    type Driver = super::Postgres;

    async fn sql_request(&mut self, _sql: &Sql) -> anyhow::Result<()> {
        Ok(())
    }
}
