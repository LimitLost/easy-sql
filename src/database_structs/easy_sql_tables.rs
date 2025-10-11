use std::fmt::Debug;

use crate::{Driver, DriverConnection, SqlOutput};
use easy_macros::macros::always_context;
use sql_compilation_data::SqlType;
use sql_macros::{SqlInsert, SqlUpdate, sql_convenience};

use crate::{DatabaseSetup, EasyExecutor, SqlTable, TableField, TableJoin};

#[derive(SqlInsert, Debug)]
#[sql(table = EasySqlTables)]
pub struct EasySqlTables {
    pub table_id: String,
    pub version: i64,
}
#[macro_export]
macro_rules! EasySqlTables_create {
    ($driver:path, $conn:expr, $table_id:expr, $version:expr) => {
        let inserted = $crate::EasySqlTables {
            table_id: $table_id,
            version: $version,
        };
        <$crate::EasySqlTables as $crate::SqlTable<$driver>>::insert($conn, &inserted)
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
        #[derive($crate::SqlUpdate, $crate::SqlOutput, Debug)]
        #[sql(table = $crate::EasySqlTables)]
        struct EasySqlTableVersion {
            pub version: i64,
        }

        <$crate::EasySqlTables as $crate::SqlTable<$driver>>::update(
            $conn,
            &mut EasySqlTableVersion {
                version: $new_version,
            },
            $crate::sql!(table_id = { $table_id }),
        )
        .await?;
    }};
}

#[macro_export]
macro_rules! EasySqlTables_get_version {
    ($driver:path, $conn:expr, $table_id:expr) => {{
        #[derive($crate::SqlUpdate, $crate::SqlOutput, Debug)]
        #[sql(table = $crate::EasySqlTables)]
        struct EasySqlTableVersion {
            pub version: i64,
        }

        let version: Option<EasySqlTableVersion> =
            <$crate::EasySqlTables as $crate::SqlTable<$driver>>::select(
                $conn,
                $crate::sql!(|$crate::EasySqlTables| table_id = { $table_id }),
            )
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
        (): ToConvert<D> + SqlOutput<Self, D, ()>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        let inserted = EasySqlTables { table_id, version };
        <EasySqlTables as SqlTable<D>>::insert(conn, &inserted).await?;

        Ok(())
    } */

    /*pub async fn update_version<D: Driver>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        table_id: &str,
        new_version: i64,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + SqlOutput<Self, D, ()>,
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
            <EasySqlTables as SqlTable<D>>::select(conn, sql_where!(table_id = { table_id }))
                .await?;
        Ok(version.map(|v| v.version))
    }*/
}

#[derive(SqlUpdate, SqlOutput, Debug)]
#[sql(table = EasySqlTables)]
struct EasySqlTableVersion {
    pub version: i64,
}

#[always_context]
impl<D: Driver + 'static> SqlTable<D> for EasySqlTables
where
    DriverConnection<D>: Send + Sync,
{
    fn table_name() -> &'static str {
        "easy_sql_tables"
    }

    fn primary_keys() -> std::vec::Vec<&'static str> {
        vec!["table_id"]
    }

    fn table_joins() -> Vec<TableJoin<'static, D>> {
        vec![]
    }
}

#[always_context]
impl<D: Driver + 'static> DatabaseSetup<D> for EasySqlTables
where
    DriverConnection<D>: Send + Sync,
{
    async fn setup(conn: &mut (impl EasyExecutor<D> + Send + Sync)) -> anyhow::Result<()> {
        use anyhow::Context;

        let table_name = <EasySqlTables as SqlTable<D>>::table_name();

        let table_exists = D::table_exists(conn, table_name).await?;

        if !table_exists {
            D::create_table(
                conn,
                table_name,
                vec![
                    TableField::<D> {
                        name: "table_id",
                        data_type: SqlType::String,
                        is_unique: false,
                        is_not_null: true,
                        default: None,
                    },
                    TableField::<D> {
                        name: "version",
                        data_type: SqlType::I64,
                        is_unique: false,
                        is_not_null: true,
                        default: None,
                    },
                ],
                vec!["table_id"],
                false,
                #[context(no)]
                Default::default(),
            )
            .await?;
        }

        Ok(())
    }
}
