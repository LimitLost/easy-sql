use std::collections::HashMap;

use crate::{Driver, EasyExecutor, driver::TableField};
use anyhow::Result;
use easy_macros::always_context;
use sql_macros::{impl_supports_fn, impl_supports_fn_any};

#[cfg(feature = "postgres")]
type ExampleInternal = sqlx::Postgres;

#[cfg(all(not(feature = "postgres"), feature = "sqlite"))]
type ExampleInternal = sqlx::Sqlite;

#[derive(Debug)]
struct DocDriver;

#[always_context]
impl Driver for DocDriver {
    type InternalDriver = ExampleInternal;

    fn identifier_delimiter() -> &'static str {
        "\""
    }

    fn parameter_placeholder(index: usize) -> String {
        format!("?{}", index + 1)
    }

    async fn table_exists(
        _conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        _name: &'static str,
    ) -> Result<bool> {
        todo!()
    }

    async fn create_table(
        _conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        _table_name: &'static str,
        _fields: Vec<TableField>,
        _primary_keys: Vec<&'static str>,
        _foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> Result<()> {
        todo!()
    }
}

#[docify::export_content]
#[allow(dead_code)]
#[allow(non_local_definitions)]
fn impl_supports_fn_basic_example() {
    impl_supports_fn!(DocDriver, SupportsCount, 0, 1);
    impl_supports_fn!(DocDriver, SupportsCurrentTimestamp, -1);
}

#[docify::export_content]
#[allow(dead_code)]
#[allow(non_local_definitions)]
fn impl_supports_fn_any_example() {
    impl_supports_fn_any!(DocDriver, SupportsConcat);
}
