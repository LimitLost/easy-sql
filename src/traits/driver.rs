use std::{collections::HashMap, fmt::Debug};

use easy_macros::macros::always_context;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::Encode;

use crate::{EasyExecutor, TableField};

pub type DriverRow<D: Driver> = <D::InternalDriver as sqlx::database::Database>::Row;

pub type DriverConnection<D: Driver> = <D::InternalDriver as sqlx::database::Database>::Connection;
pub type DriverArguments<'a, D: Driver> =
    <D::InternalDriver as sqlx::database::Database>::Arguments<'a>;
pub type InternalDriver<DI> = <DI as Driver>::InternalDriver;

#[always_context]
pub trait Driver: Debug + Send + Sync + Sized {
    type InternalDriver: sqlx::Database;
    type Value<'a>: DriverValue<'a, Self::InternalDriver>;

    async fn table_exists(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        name: &'static str,
    ) -> anyhow::Result<bool>;

    /// `foreign_keys` - Key - table name
    ///
    /// `foreign_keys` - Value - field names, foreign field names, on delete/update cascade
    async fn create_table<'a>(
        conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        table_name: &'static str,
        fields: Vec<TableField<'a, Self>>,
        primary_keys: Vec<&'static str>,
        auto_increment: bool,
        foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> anyhow::Result<()>;

    fn binary_value(bytes: Vec<u8>) -> Self::Value<'static>;
}

#[always_context]
pub trait DriverValue<'a, IDriver: sqlx::Database>:
    Encode<'a, IDriver> + sqlx::Type<IDriver> + DeserializeOwned + Serialize + Debug + Send + Sync
{
    fn to_default(&self) -> anyhow::Result<String>;
}
