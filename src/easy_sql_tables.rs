use std::fmt::Debug;

use anyhow::Context;
use async_trait::async_trait;
use easy_macros::macros::always_context;
use sql_macros::{SqlInsert, SqlOutput, SqlUpdate, sql_where};

use crate::{
    CreateTable, DatabaseSetup, EasyExecutor, SqlTable, SqlTableTest, SqlType, TableExists,
    TableField,
};

#[derive(SqlInsert, Debug)]
#[sql(table = EasySqlTables)]
pub struct EasySqlTables {
    pub table_id: String,
    pub version: i64,
}

#[always_context]
//TODO Make this (#[sql]) a part of sql build function
#[sql]
impl EasySqlTables {
    pub async fn create(
        conn: &mut (impl EasyExecutor + Send + Sync),
        table_id: String,
        version: i64,
    ) -> anyhow::Result<()> {
        EasySqlTables::insert(conn, &EasySqlTables { table_id, version }).await?;

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
            Some(crate::WhereClause {
                conditions: crate::WhereExpr::Eq(
                    Box::new(crate::WhereExpr::Column("table_id".to_string())),
                    Box::new(crate::WhereExpr::Value({ table_id }.into())),
                ),
            }), // sql_where!(table_id = { table_id }),
            SqlTableTest::new(|___t___| {
                let _ = ___t___.table_id;
            }),
        )
        .await?;

        Ok(())
    }

    pub async fn get_version(
        conn: &mut (impl EasyExecutor + Send + Sync),
        table_id: &str,
    ) -> anyhow::Result<i64> {
        #[no_context]
        let version: EasySqlTableVersion =
            EasySqlTables::select(conn, sql_where!(table_id = { table_id })).await?;
        Ok(version.version)
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
}

#[always_context]
#[async_trait]
impl DatabaseSetup for EasySqlTables {
    async fn setup(
        conn: &mut (impl EasyExecutor + Send + Sync),
        used_table_names: &mut Vec<String>,
    ) -> anyhow::Result<()> {
        use crate::EasyExecutor;
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
                        name: "table_id".to_string(),
                        data_type: SqlType::String,
                        is_primary_key: true,
                        foreign_key: None,
                        is_unique: false,
                        is_not_null: true,
                    },
                    TableField {
                        name: "version".to_string(),
                        data_type: SqlType::I64,
                        is_primary_key: false,
                        foreign_key: None,
                        is_unique: false,
                        is_not_null: true,
                    },
                ],
            })
            .await?;
        }

        Ok(())
    }
}
