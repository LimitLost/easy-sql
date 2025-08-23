use std::fmt::Debug;

use anyhow::Context;
use async_trait::async_trait;
use easy_macros::macros::always_context;
use sql_compilation_data::SqlType;
use sql_macros::{SqlInsert, SqlOutput, SqlUpdate, sql_convenience, sql_where};

use crate::{
    CreateTable, DatabaseSetup, EasyExecutor, SqlTable, TableExists, TableField, TableJoin,
};

#[derive(SqlInsert, Debug)]
#[sql(table = EasySqlTables)]
pub struct EasySqlTables {
    pub table_id: String,
    pub version: i64,
}

#[sql_convenience]
#[always_context]
impl EasySqlTables {
    pub async fn create(
        conn: &mut (impl EasyExecutor + Send + Sync),
        table_id: String,
        version: i64,
    ) -> anyhow::Result<()> {
        let inserted = EasySqlTables { table_id, version };
        EasySqlTables::insert(conn, &inserted).await?;

        Ok(())
    }

    pub async fn update_version(
        conn: &mut (impl EasyExecutor + Send + Sync),
        table_id: &str,
        new_version: i64,
    ) -> anyhow::Result<()> {
        EasySqlTables::update(
            conn,
            EasySqlTableVersion {
                version: new_version,
            },
            sql_where!(table_id = { table_id }),
        )
        .await?;

        Ok(())
    }

    pub async fn get_version(
        conn: &mut (impl EasyExecutor + Send + Sync),
        table_id: &str,
    ) -> anyhow::Result<Option<i64>> {
        #[no_context]
        let version: Option<EasySqlTableVersion> =
            EasySqlTables::select(conn, sql_where!(table_id = { table_id })).await?;
        Ok(version.map(|v| v.version))
    }
}

#[derive(SqlUpdate, SqlOutput, Debug)]
#[sql(table = EasySqlTables)]
struct EasySqlTableVersion {
    pub version: i64,
}

#[always_context]
impl SqlTable for EasySqlTables {
    fn table_name() -> &'static str {
        "easy_sql_tables"
    }

    fn primary_keys() -> std::vec::Vec<&'static str> {
        vec!["table_id"]
    }

    fn table_joins() -> Vec<TableJoin<'static>> {
        vec![]
    }
}

#[always_context]
#[async_trait]
impl DatabaseSetup for EasySqlTables {
    async fn setup(conn: &mut (impl EasyExecutor + Send + Sync)) -> anyhow::Result<()> {
        use anyhow::Context;

        let table_exists = conn
            .query_setup(TableExists {
                name: EasySqlTables::table_name(),
            })
            .await?;

        if !table_exists {
            conn.query_setup(CreateTable {
                table_name: EasySqlTables::table_name(),
                fields: vec![
                    TableField {
                        name: "table_id",
                        data_type: SqlType::String,
                        is_unique: false,
                        is_not_null: true,
                        default: None,
                    },
                    TableField {
                        name: "version",
                        data_type: SqlType::I64,
                        is_unique: false,
                        is_not_null: true,
                        default: None,
                    },
                ],
                primary_keys: vec!["table_id"],
                auto_increment: false,
                foreign_keys: Default::default(),
            })
            .await?;
        }

        Ok(())
    }
}
