use std::fmt::Debug;

use crate::{Driver, DriverConnection, HasTable, InternalDriver, Output, QueryBuilder};
use easy_macros::always_context;
use sql_macros::{Insert, Update, sql_convenience};
use sqlx::TypeInfo;

use crate::{DatabaseSetup, EasyExecutor, Table, TableField, TableJoin};

#[derive(Insert, Debug)]
#[sql(table = EasySqlTables)]
pub struct EasySqlTables {
    pub table_id: String,
    pub version: i64,
}

impl HasTable<EasySqlTables> for EasySqlTables {}

#[macro_export]
macro_rules! EasySqlTables_create {
    ($driver:path, $conn:expr, $table_id:expr, $version:expr) => {
        let inserted = $crate::EasySqlTables {
            table_id: $table_id,
            version: $version,
        };
        $crate::query!($conn, INSERT INTO $crate::EasySqlTables VALUES { &inserted })
        .await
        .with_context($crate::macro_support::context!(
            "Failed to create EasySqlTables | inserted: {:?}",
            inserted
        ))?;
    };
}

#[macro_export]
macro_rules! EasySqlTables_update_version {
    ($driver:path, $conn:expr, $table_id:expr, $new_version:expr) => {{

        $crate::query!($conn, UPDATE $crate::EasySqlTables SET version = { $new_version } WHERE table_id = { $table_id })
            .await
            .with_context($crate::macro_support::context!(
                "Failed to update EasySqlTables version | table_id: {:?} | new_version: {:?}",
                $table_id,
                $new_version
            ))?;
    }};
}

#[macro_export]
macro_rules! EasySqlTables_get_version {
    ($driver:path, $conn:expr, $table_id:expr) => {{
        #[derive($crate::Update, $crate::Output, Debug)]
        #[sql(table = $crate::EasySqlTables)]
        struct EasySqlTableVersion {
            pub version: i64,
        }

        let version: Option<EasySqlTableVersion> = $crate::query!($conn, SELECT Option<EasySqlTableVersion> FROM $crate::EasySqlTables WHERE $crate::EasySqlTables.table_id = { $table_id })
            .await?;
        version.map(|v| v.version)
    }};
}

#[sql_convenience]
#[always_context]
impl EasySqlTables {
    /* pub async fn create<D: Driver>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        table_id: String,
        version: i64,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + Output<Self, D, ()>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        let inserted = EasySqlTables { table_id, version };
        <EasySqlTables as Table<D>>::insert(conn, &inserted).await?;

        Ok(())
    } */

    /*pub async fn update_version<D: Driver>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        table_id: &str,
        new_version: i64,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + Output<Self, D, ()>,
        DriverRow<D>: ToConvert<D>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        EasySqlTables::update(
            conn,
            &mut EasySqlTableVersion {
                version: new_version,
            },
            sql_where!(table_id = { table_id }),
        )
        .await?;

        Ok(())
    }

    pub async fn get_version<D>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        table_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        #[no_context]
        let version: Option<EasySqlTableVersion> =
            <EasySqlTables as Table<D>>::select(conn, sql_where!(table_id = { table_id }))
                .await?;
        Ok(version.map(|v| v.version))
    }*/
}

#[derive(Update, Output, Debug)]
#[sql(table = EasySqlTables)]
struct EasySqlTableVersion {
    pub version: i64,
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
        .await?;

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
            .await?;
        }

        Ok(())
    }
}
