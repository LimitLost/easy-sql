use std::collections::HashMap;

use anyhow::Context;
use easy_macros::macros::always_context;

mod sql_value;
pub use sql_value::*;
mod database_internal_default;
pub use database_internal_default::*;
mod database;
pub use database::*;

mod to_convert_impl;

mod alter_table;
pub use alter_table::*;
mod create_table;
pub use create_table::*;
mod table_exists;
pub use table_exists::*;

use crate::{Driver, EasyExecutor, TableField};

#[derive(Debug)]
pub struct Sqlite;

type Db = sqlx::Sqlite;

#[always_context]
impl Driver for Sqlite {
    type InternalDriver = Db;
    type Value<'a> = sql_value::SqlValueMaybeRef<'a>;

    async fn table_exists(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        name: &'static str,
    ) -> anyhow::Result<bool> {
        let result = conn.query_setup(TableExists { name }).await?;
        Ok(result)
    }

    /// `auto_increment` - Can only be used when with single primary key
    ///
    /// `foreign_keys` - Key - table name
    ///
    /// `foreign_keys` - Value - field names, foreign field names, on delete/update cascade
    #[no_context_inputs]
    async fn create_table<'a>(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        table_name: &'static str,
        fields: Vec<TableField<'a, Self>>,

        primary_keys: Vec<&'static str>,
        auto_increment: bool,
        foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> anyhow::Result<()> {
        conn.query_setup(CreateTable {
            table_name,
            fields,
            primary_keys,
            auto_increment,
            foreign_keys,
        })
        .await?;
        Ok(())
    }

    fn binary_value(bytes: Vec<u8>) -> Self::Value<'static> {
        SqlValueMaybeRef::Value(SqlValue::Bytes(bytes))
    }
}
