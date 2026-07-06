use std::fmt::Debug;

use crate::{
    Driver,
    markers::{HasTable, NotJoinedTable},
    traits::{DriverConnection, InternalDriver},
};
use easy_macros::always_context;
use easy_sql_macros::Insert;
use sqlx::TypeInfo;

use crate::{DatabaseSetup, EasyExecutor, Table, driver::TableField};

/// Internal metadata table used by Easy SQL to track per-table schema versions.
///
/// This table is created automatically via [`DatabaseSetup`](macro@crate::DatabaseSetup) and is used by the
/// [`Table`](macro@crate::Table) derive macro to store the last known version of each table.
/// It lives in the database as `easy_sql_tables`, keyed by `table_id`.
#[derive(Insert, Debug)]
#[sql(table = EasySqlTables)]
pub struct EasySqlTables {
    /// The logical identifier of the table being tracked.
    pub table_id: String,
    /// The stored schema version for the table.
    pub version: i64,
}

impl HasTable<EasySqlTables> for EasySqlTables {}

impl NotJoinedTable for EasySqlTables {}

#[macro_export]
#[doc(hidden)]
/// Used by Table derive macro
macro_rules! EasySqlTables_create {
    ($driver:path, $conn:expr, $table_id:expr, $version:expr) => {
        use $crate::macro_support::Context;
        let table_id_value = $table_id;
        let version_value = $version;
        let inserted = $crate::EasySqlTables {
            table_id: table_id_value,
            version: version_value,
        };
        $crate::query!($conn, INSERT INTO $crate::EasySqlTables VALUES { &inserted })
        .await
        .with_context($crate::macro_support::context!(
            "setup failed: operation=create_version_row, metadata_table=easy_sql_tables, table_id={:?}, version={}, driver={}, inserted={:?}",
            inserted.table_id,
            inserted.version,
            stringify!($driver),
            inserted
        ))?;
    };
}

#[macro_export]
#[doc(hidden)]
/// Used by Table derive macro
macro_rules! EasySqlTables_update_version {
    ($driver:path, $conn:expr, $table_id:expr, $new_version:expr) => {{
        use $crate::macro_support::Context;
        let table_id_value = $table_id;
        let new_version_value = $new_version;

        $crate::query!($conn, UPDATE $crate::EasySqlTables SET version = { new_version_value } WHERE table_id = { table_id_value })
            .await
            .with_context($crate::macro_support::context!(
                "setup failed: operation=update_version_row, metadata_table=easy_sql_tables, table_id={:?}, new_version={}, driver={}",
                table_id_value,
                new_version_value,
                stringify!($driver)
            ))?;
    }};
}

#[macro_export]
#[doc(hidden)]
/// Used by Table derive macro
macro_rules! EasySqlTables_get_version {
    ($driver:path, $conn:expr, $table_id:expr) => {{
        use $crate::macro_support::Context;
        let table_id_value = $table_id;
        let table_id_for_query = table_id_value.clone();

        #[derive($crate::Update, $crate::Output, Debug)]
        #[sql(table = $crate::EasySqlTables)]
        struct EasySqlTableVersion {
            pub version: i64,
        }

        let version: Option<EasySqlTableVersion> = $crate::query!($conn, SELECT Option<EasySqlTableVersion> FROM $crate::EasySqlTables WHERE $crate::EasySqlTables.table_id = { table_id_for_query })
            .await
            .with_context($crate::macro_support::context!(
                "setup failed: operation=get_version, metadata_table=easy_sql_tables, table_id={:?}, driver={}",
                table_id_value,
                stringify!($driver)
            ))?;
        version.map(|v| v.version)
    }};
}

#[always_context]
impl<D: Driver + 'static> Table<D> for EasySqlTables
where
    DriverConnection<D>: Send + Sync,
{
    fn table_name() -> &'static str {
        "easy_sql_tables"
    }

    fn primary_keys() -> std::vec::Vec<&'static str> {
        vec!["table_id"]
    }

    fn table_joins(_current_query: &mut String) {}
}

#[always_context]
impl<D: Driver + 'static> DatabaseSetup<D> for EasySqlTables
where
    DriverConnection<D>: Send + Sync,
    String: sqlx::Type<InternalDriver<D>>,
    i64: sqlx::Type<InternalDriver<D>>,
{
    async fn setup(conn: &mut (impl EasyExecutor<D> + Send + Sync)) -> anyhow::Result<()> {
        use anyhow::Context;

        let table_name = <EasySqlTables as Table<D>>::table_name();

        let table_exists = D::table_exists(
            #[context(no)]
            conn,
            table_name,
        )
        .await
        .with_context(|| format!("driver={}", std::any::type_name::<D>()))?;

        if !table_exists {
            D::create_table(
                #[context(no)]
                conn,
                table_name,
                vec![
                    TableField {
                        name: "table_id",
                        data_type: <String as sqlx::Type<InternalDriver<D>>>::type_info()
                            .name()
                            .to_owned(),
                        is_unique: false,
                        is_not_null: true,
                        default: None,
                        is_auto_increment: false,
                    },
                    TableField {
                        name: "version",
                        data_type: <i64 as sqlx::Type<InternalDriver<D>>>::type_info()
                            .name()
                            .to_owned(),
                        is_unique: false,
                        is_not_null: true,
                        default: None,
                        is_auto_increment: false,
                    },
                ],
                vec!["table_id"],
                #[context(no)]
                Default::default(),
            )
            .await
            .with_context(|| format!("driver={}", std::any::type_name::<D>()))?;
        }

        Ok(())
    }
}
