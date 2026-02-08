mod alter_table;
mod create_table;
mod database;
mod table_exists;
use create_table::CreateTable;
pub use database::*;
use table_exists::TableExists;

mod connection;
mod to_convert_impl;
mod to_default_impl;

use std::collections::HashMap;

use anyhow::Context;
use easy_macros::always_context;

use crate::{
    Driver, EasyExecutor,
    driver::TableField,
    markers::{
        AllowsNoPrimaryKey, SupportsAutoIncrementCompositePrimaryKey,
        operators::{
            SupportsAdd, SupportsAnd, SupportsBetween, SupportsBitAnd, SupportsBitOr,
            SupportsBitShiftLeft, SupportsBitShiftRight, SupportsConcatOperator, SupportsDiv,
            SupportsEqual, SupportsGreaterThan, SupportsGreaterThanOrEqual, SupportsIn,
            SupportsIsNotNull, SupportsIsNull, SupportsJsonExtract, SupportsJsonExtractText,
            SupportsLessThan, SupportsLessThanOrEqual, SupportsLike, SupportsModOperator,
            SupportsMul, SupportsNotEqual, SupportsOr, SupportsSub,
        },
    },
};
use sql_macros::{impl_supports_fn, impl_supports_fn_any};

/// Marker type for the PostgreSQL driver.
///
/// Use as the driver parameter in macros when explicit selection is needed.
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

#[always_context]
impl AllowsNoPrimaryKey for Postgres {}

#[always_context]
impl SupportsAutoIncrementCompositePrimaryKey for Postgres {}

impl_supports_fn!(Postgres, SupportsCount, 0, 1);
impl_supports_fn!(Postgres, SupportsSum, 1);
impl_supports_fn!(Postgres, SupportsAvg, 1);
impl_supports_fn!(Postgres, SupportsMin, 1);
impl_supports_fn!(Postgres, SupportsMax, 1);

impl_supports_fn_any!(Postgres, SupportsConcat);
impl_supports_fn!(Postgres, SupportsUpper, 1);
impl_supports_fn!(Postgres, SupportsLower, 1);
impl_supports_fn!(Postgres, SupportsLength, 1);
impl_supports_fn!(Postgres, SupportsTrim, 1);
impl_supports_fn!(Postgres, SupportsSubstring, 2, 3);
impl_supports_fn!(Postgres, SupportsSubstr, 2, 3);

impl_supports_fn_any!(Postgres, SupportsCoalesce);
impl_supports_fn!(Postgres, SupportsNullif, 2);

impl_supports_fn!(Postgres, SupportsNow, 0);
impl_supports_fn!(Postgres, SupportsDate, 1);
impl_supports_fn!(Postgres, SupportsTime, 1);
impl_supports_fn!(Postgres, SupportsCurrentTimestamp, -1);
impl_supports_fn!(Postgres, SupportsCurrentDate, -1);
impl_supports_fn!(Postgres, SupportsCurrentTime, -1);

impl_supports_fn!(Postgres, SupportsAbs, 1);
impl_supports_fn!(Postgres, SupportsRound, 1, 2);
impl_supports_fn!(Postgres, SupportsMod, 2);
impl_supports_fn!(Postgres, SupportsCeil, 1);
impl_supports_fn!(Postgres, SupportsCeiling, 1);
impl_supports_fn!(Postgres, SupportsFloor, 1);
impl_supports_fn!(Postgres, SupportsPower, 2);
impl_supports_fn!(Postgres, SupportsPow, 2);
impl_supports_fn!(Postgres, SupportsSqrt, 1);

impl_supports_fn!(Postgres, SupportsCast, 1, 2);
impl_supports_fn!(Postgres, SupportsDistinct, 1);

impl SupportsAnd for Postgres {}
impl SupportsOr for Postgres {}
impl SupportsAdd for Postgres {}
impl SupportsSub for Postgres {}
impl SupportsMul for Postgres {}
impl SupportsDiv for Postgres {}
impl SupportsModOperator for Postgres {}
impl SupportsConcatOperator for Postgres {}
impl SupportsJsonExtract for Postgres {}
impl SupportsJsonExtractText for Postgres {}
impl SupportsBitAnd for Postgres {}
impl SupportsBitOr for Postgres {}
impl SupportsBitShiftLeft for Postgres {}
impl SupportsBitShiftRight for Postgres {}
impl SupportsEqual for Postgres {}
impl SupportsNotEqual for Postgres {}
impl SupportsGreaterThan for Postgres {}
impl SupportsGreaterThanOrEqual for Postgres {}
impl SupportsLessThan for Postgres {}
impl SupportsLessThanOrEqual for Postgres {}
impl SupportsLike for Postgres {}
impl SupportsIsNull for Postgres {}
impl SupportsIsNotNull for Postgres {}
impl SupportsIn for Postgres {}
impl SupportsBetween for Postgres {}

fn table_field_definition(field: TableField) -> String {
    let TableField {
        name,
        data_type,
        is_unique,
        is_not_null,
        default,
        is_auto_increment,
    } = field;

    let unique = if is_unique { "UNIQUE" } else { "" };
    // IDENTITY columns already imply NOT NULL, so we don't need to add it
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

    // Handle auto-increment types using identity columns
    let data_type = if is_auto_increment {
        let base_type = match data_type.to_uppercase().as_str() {
            "SMAILLINT" | "INT2" => "SMALLINT",
            "INTEGER" | "INT" | "INT4" => "INTEGER",
            "BIGINT" | "INT8" => "BIGINT",
            _ => data_type.as_str(),
        };
        format!("{base_type} GENERATED BY DEFAULT AS IDENTITY")
    } else {
        data_type
    };

    format!(
        "\"{}\" {} {} {} {},",
        name, data_type, unique, not_null, default
    )
}
