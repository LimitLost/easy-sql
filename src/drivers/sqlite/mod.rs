use std::collections::HashMap;

use anyhow::Context;
use create_table::CreateTable;
use easy_macros::always_context;

mod database;
pub use database::*;
use table_exists::TableExists;

mod connection;
mod to_convert_impl;
mod to_default_impl;

mod alter_table;
mod create_table;
mod table_exists;

use crate::{
    Driver, EasyExecutor, TableField,
    markers::{
        AllowsNoPrimaryKey, SupportsAdd, SupportsAnd, SupportsBetween, SupportsBitAnd,
        SupportsBitOr, SupportsBitShiftLeft, SupportsBitShiftRight, SupportsConcatOperator,
        SupportsDiv, SupportsEqual, SupportsGreaterThan, SupportsGreaterThanOrEqual, SupportsIn,
        SupportsIsNotNull, SupportsIsNull, SupportsLessThan, SupportsLessThanOrEqual, SupportsLike,
        SupportsModOperator, SupportsMul, SupportsNotEqual, SupportsOr, SupportsSub,
    },
};
use sql_macros::{impl_supports_fn, impl_supports_fn_any};

/// Marker type for the SQLite driver.
///
/// Use as the driver parameter in macros when explicit selection is needed.
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
impl AllowsNoPrimaryKey for Sqlite {}

impl_supports_fn!(Sqlite, SupportsCount, 0, 1);
impl_supports_fn!(Sqlite, SupportsSum, 1);
impl_supports_fn!(Sqlite, SupportsAvg, 1);
impl_supports_fn!(Sqlite, SupportsMin, 1);
impl_supports_fn!(Sqlite, SupportsMax, 1);

impl_supports_fn_any!(Sqlite, SupportsConcat);
impl_supports_fn!(Sqlite, SupportsUpper, 1);
impl_supports_fn!(Sqlite, SupportsLower, 1);
impl_supports_fn!(Sqlite, SupportsLength, 1);
impl_supports_fn!(Sqlite, SupportsTrim, 1);
impl_supports_fn!(Sqlite, SupportsSubstring, 2, 3);
impl_supports_fn!(Sqlite, SupportsSubstr, 2, 3);

impl_supports_fn_any!(Sqlite, SupportsCoalesce);
impl_supports_fn!(Sqlite, SupportsNullif, 2);
impl_supports_fn!(Sqlite, SupportsIfnull, 2);

impl_supports_fn!(Sqlite, SupportsDate, 1);
impl_supports_fn!(Sqlite, SupportsTime, 1);
impl_supports_fn!(Sqlite, SupportsDatetime, 1);
impl_supports_fn!(Sqlite, SupportsCurrentTimestamp, -1);
impl_supports_fn!(Sqlite, SupportsCurrentDate, -1);
impl_supports_fn!(Sqlite, SupportsCurrentTime, -1);

impl_supports_fn!(Sqlite, SupportsAbs, 1);
impl_supports_fn!(Sqlite, SupportsRound, 1, 2);
#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsMod, 2);

#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsCeil, 1);
#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsCeiling, 1);
#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsFloor, 1);
#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsPower, 2);
#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsPow, 2);
#[cfg(feature = "sqlite_math")]
impl_supports_fn!(Sqlite, SupportsSqrt, 1);

impl_supports_fn!(Sqlite, SupportsCast, 1, 2);
impl_supports_fn!(Sqlite, SupportsDistinct, 1);

impl SupportsAnd for Sqlite {}
impl SupportsOr for Sqlite {}
impl SupportsAdd for Sqlite {}
impl SupportsSub for Sqlite {}
impl SupportsMul for Sqlite {}
impl SupportsDiv for Sqlite {}
impl SupportsModOperator for Sqlite {}
impl SupportsConcatOperator for Sqlite {}
impl SupportsBitAnd for Sqlite {}
impl SupportsBitOr for Sqlite {}
impl SupportsBitShiftLeft for Sqlite {}
impl SupportsBitShiftRight for Sqlite {}
impl SupportsEqual for Sqlite {}
impl SupportsNotEqual for Sqlite {}
impl SupportsGreaterThan for Sqlite {}
impl SupportsGreaterThanOrEqual for Sqlite {}
impl SupportsLessThan for Sqlite {}
impl SupportsLessThanOrEqual for Sqlite {}
impl SupportsLike for Sqlite {}
impl SupportsIsNull for Sqlite {}
impl SupportsIsNotNull for Sqlite {}
impl SupportsIn for Sqlite {}
impl SupportsBetween for Sqlite {}

#[always_context]
fn table_field_definition(field: TableField) -> String {
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
