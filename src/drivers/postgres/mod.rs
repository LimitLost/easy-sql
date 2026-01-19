mod table_exists;
pub use table_exists::*;
mod create_table;
pub use create_table::*;
mod alter_table;
mod database;
pub use database::*;

mod connection;
mod to_convert_impl;
mod to_default_impl;

use std::collections::HashMap;

use anyhow::Context;
use easy_macros::always_context;

use crate::{Driver, EasyExecutor, TableField};

#[derive(Debug)]
pub struct Postgres;

type Db = sqlx::Postgres;

#[always_context]
impl Driver for Postgres {
    type InternalDriver = Db;
    fn identifier_delimiter() -> &'static str {
        "\""
    }

    fn parameter_placeholder(index: usize) -> String {
        format!("${}", index + 1)
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
            fields,
            primary_keys,
            foreign_keys,
        })
        .await?;
        Ok(())
    }
}

pub fn table_field_definition(field: TableField) -> String {
    let TableField {
        name,
        data_type,
        is_unique,
        is_not_null,
        default,
        is_auto_increment,
    } = field;

    let unique = if is_unique { "UNIQUE" } else { "" };
    // SERIAL/BIGSERIAL already implies NOT NULL, so we don't need to add it
    let not_null = if is_not_null && !is_auto_increment {
        "NOT NULL"
    } else {
        ""
    };
    let default = if let Some(default) = default {
        if is_auto_increment {
            // SERIAL/BIGSERIAL already has a default sequence, don't override
            String::new()
        } else {
            format!("DEFAULT {}", default)
        }
    } else {
        String::new()
    };

    // Handle auto-increment types
    let data_type = if is_auto_increment {
        match data_type.to_uppercase().as_str() {
            "SMAILLINT" | "INT2" => "SMALLSERIAL".to_string(),
            "INTEGER" | "INT" | "INT4" => "SERIAL".to_string(),
            "BIGINT" | "INT8" => "BIGSERIAL".to_string(),
            _ => data_type,
        }
    } else {
        data_type
    };

    format!(
        "\"{}\" {} {} {} {},",
        name, data_type, unique, not_null, default
    )
}
