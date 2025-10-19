mod sql_value;
mod table_exists;
pub use table_exists::*;
mod create_table;
pub use create_table::*;
mod database;
pub use database::*;
mod database_internal_default;
pub use database_internal_default::*;
mod to_convert_impl;

use std::collections::HashMap;

use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};

use crate::{
    Driver, DriverValue, EasyExecutor, TableField,
    general_value::{SqlValue, SqlValueMaybeRef},
};

#[derive(Debug)]
pub struct Postgres;

type Db = sqlx::Postgres;

#[always_context]
impl Driver for Postgres {
    type InternalDriver = Db;
    type Value<'a> = SqlValueMaybeRef<'a>;
    fn identifier_delimiter() -> &'static str {
        "\""
    }

    fn parameter_placeholder(index: usize) -> String {
        format!("${}", index + 1)
    }

    fn binary_value(bytes: Vec<u8>) -> Self::Value<'static> {
        SqlValueMaybeRef::Value(SqlValue::Bytes(bytes))
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
    async fn create_table<'a>(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        table_name: &'static str,
        fields: Vec<TableField<'a, Self>>,
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

#[always_context]
impl<'a> TableField<'a, Postgres> {
    pub fn definition(self) -> anyhow::Result<String> {
        use easy_macros::helpers::context;

        let TableField {
            name,
            data_type,
            is_unique,
            is_not_null,
            default,
            is_auto_increment,
        } = self;

        let data_type_str = data_type.postgres(is_auto_increment);

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
                format!(
                    "DEFAULT {}",
                    default
                        .to_default()
                        .with_context(context!("field name: {}", name))?
                )
            }
        } else {
            String::new()
        };

        Ok(format!(
            "\"{}\" {} {} {} {},",
            name, data_type_str, unique, not_null, default
        ))
    }
}
