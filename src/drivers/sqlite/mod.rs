use std::collections::HashMap;

use anyhow::Context;
use easy_macros::always_context;

mod database;
pub use database::*;

mod connection;
mod to_convert_impl;
mod to_default_impl;

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
    fn identifier_delimiter() -> &'static str {
        "`"
    }

    fn parameter_placeholder(index: usize) -> String {
        format!("?{}", index + 1)
    }

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
    async fn create_table(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        table_name: &'static str,
        fields: Vec<TableField>,
        primary_keys: Vec<&'static str>,
        foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> anyhow::Result<()> {
        conn.query_setup(CreateTable {
            table_name,
            auto_increment: fields.iter().any(|f| f.is_auto_increment),
            fields,
            primary_keys,
            foreign_keys,
        })
        .await?;
        Ok(())
    }
}

#[always_context]
pub fn table_field_definition(field: TableField) -> String {
    let TableField {
        name,
        data_type,
        is_unique,
        is_not_null,
        default,
        is_auto_increment: _, // SQLite handles auto_increment in PRIMARY KEY constraint
    } = field;

    let unique = if is_unique { "UNIQUE" } else { "" };
    let not_null = if is_not_null { "NOT NULL" } else { "" };
    let default = if let Some(default) = default {
        format!("DEFAULT {}", default)
    } else {
        String::new()
    };

    format!(
        "{} {} {} {} {},",
        name, data_type, unique, not_null, default
    )
}
