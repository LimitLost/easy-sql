use std::{collections::HashMap, fmt::Debug};

use easy_macros::always_context;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::Encode;

use crate::{EasyExecutor, TableField};

pub type DriverRow<D> = <<D as Driver>::InternalDriver as sqlx::database::Database>::Row;

pub type DriverConnection<D> =
    <<D as Driver>::InternalDriver as sqlx::database::Database>::Connection;
pub type DriverArguments<'a, D> =
    <<D as Driver>::InternalDriver as sqlx::database::Database>::Arguments<'a>;
pub type InternalDriver<D> = <D as Driver>::InternalDriver;
pub type DriverQueryResult<D> =
    <<D as Driver>::InternalDriver as sqlx::database::Database>::QueryResult;
pub type DriverTypeInfo<D> = <<D as Driver>::InternalDriver as sqlx::database::Database>::TypeInfo;

#[always_context]
pub trait Driver: Debug + Send + Sync + Sized {
    type InternalDriver: sqlx::Database;

    fn identifier_delimiter() -> &'static str;

    /// index is 0 based
    fn parameter_placeholder(index: usize) -> String;

    async fn table_exists(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        name: &'static str,
    ) -> anyhow::Result<bool>;

    /// `foreign_keys` - Key - table name
    ///
    /// `foreign_keys` - Value - field names, foreign field names, on delete/update cascade
    async fn create_table(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        table_name: &'static str,
        fields: Vec<TableField>,
        primary_keys: Vec<&'static str>,
        foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> anyhow::Result<()>;
}

#[always_context]
pub trait DriverValue<'a, IDriver: sqlx::Database>:
    Encode<'a, IDriver> + sqlx::Type<IDriver> + DeserializeOwned + Serialize + Debug + Send + Sync
{
    fn to_default(&self) -> anyhow::Result<String>;
}

#[always_context]
#[diagnostic::on_unimplemented(
    message = "Driver `{Self}` does not support auto-increment columns when the table uses a composite primary key. Remove #[sql(auto_increment)] or use a single-column primary key for this driver."
)]
pub trait SupportsAutoIncrementCompositePrimaryKey: Driver {}

#[always_context]
#[diagnostic::on_unimplemented(
    message = "Driver `{Self}` requires a primary key for tables. Add #[sql(primary_key)] to at least one field."
)]
pub trait AllowsNoPrimaryKey: Driver {}

#[always_context]
#[diagnostic::on_unimplemented(
    message = "Driver `{Self}` does not support multiple auto-increment columns in the same table. Remove #[sql(auto_increment)] from all but one column."
)]
pub trait SupportsMultipleAutoIncrementColumns: Driver {}
