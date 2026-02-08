use std::{collections::HashMap, fmt::Debug};

use easy_macros::always_context;

use crate::{driver::TableField, traits::EasyExecutor};

pub type DriverRow<D> = <<D as Driver>::InternalDriver as sqlx::database::Database>::Row;

pub type DriverConnection<D> =
    <<D as Driver>::InternalDriver as sqlx::database::Database>::Connection;
pub type DriverArguments<'a, D> =
    <<D as Driver>::InternalDriver as sqlx::database::Database>::Arguments<'a>;
pub type InternalDriver<D> = <D as Driver>::InternalDriver;
pub type DriverQueryResult<D> =
    <<D as Driver>::InternalDriver as sqlx::database::Database>::QueryResult;
pub type DriverTypeInfo<D> = <<D as Driver>::InternalDriver as sqlx::database::Database>::TypeInfo;

/// Driver backend integration.
///
/// Implement this trait for custom drivers to describe the underlying `sqlx::Database` and
/// database-specific DDL helpers used by the macros and migrations.
#[always_context]
pub trait Driver: Debug + Send + Sync + Sized {
    type InternalDriver: sqlx::Database;

    fn identifier_delimiter() -> &'static str;

    /// Build a parameter placeholder for the driver (`index` is 0-based).
    fn parameter_placeholder(index: usize) -> String;

    async fn table_exists(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        name: &'static str,
    ) -> anyhow::Result<bool>;

    /// Create a table with the provided schema metadata.
    ///
    /// `foreign_keys`:
    /// - Key: referenced table name
    /// - Value: local field names, referenced field names, on delete/update cascade flag
    async fn create_table(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        table_name: &'static str,
        fields: Vec<TableField>,
        primary_keys: Vec<&'static str>,
        foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> anyhow::Result<()>;
}
