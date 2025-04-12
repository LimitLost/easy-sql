use anyhow::Context;
use async_trait::async_trait;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::Row;

use crate::SetupSql;

use super::table_field::TableField;

#[derive(Debug)]
pub struct TableExists {
    pub name: &'static str,
}

#[always_context]
#[async_trait]
impl SetupSql for TableExists {
    type Output = bool;

    async fn query<'a>(
        self,
        exec: impl sqlx::Executor<'a, Database = crate::Db> + Sync,
    ) -> anyhow::Result<Self::Output> {
        let query = format!(
            "SELECT EXISTS (SELECT * FROM sqlite_master WHERE type='table' AND name='{}')",
            self.name
        );
        #[no_context]
        let result: bool = sqlx::query(&query)
            .fetch_one(exec)
            .await
            .with_context(context!("table_name: {:?} | query: {:?}", self.name, query))?
            .get(0);
        Ok(result)
    }
}
#[derive(Debug)]
pub struct CreateTable {
    pub table_name: &'static str,
    pub fields: Vec<TableField>,
}

#[always_context]
#[async_trait]
impl SetupSql for CreateTable {
    type Output = ();

    async fn query<'a>(
        self,
        exec: impl sqlx::Executor<'a, Database = crate::Db> + Sync,
    ) -> anyhow::Result<Self::Output> {
        let mut table_fields = String::new();
        let mut table_constrains = String::new();

        for field in self.fields.into_iter() {
            let TableField {
                name,
                data_type,
                is_primary_key,
                foreign_key,
                is_unique,
                is_not_null,
            } = field;

            let date_type_sqlite = data_type.sqlite();

            let primary_key = if is_primary_key { "PRIMARY KET" } else { "" };
            let unique = if is_unique { "UNIQUE" } else { "" };
            let not_null = if is_not_null { "NOT NULL" } else { "" };

            if let Some(foreign_key) = foreign_key {
                table_constrains.push_str(&format!(
                    "FOREIGN KEY ({}) REFERENCES {}({}),",
                    name, foreign_key.table_name, foreign_key.referenced_field
                ));
            }

            table_fields.push_str(&format!(
                "{} {} {} {} {},",
                name, date_type_sqlite, primary_key, unique, not_null
            ));
        }

        if table_constrains.is_empty() && !table_fields.is_empty() {
            //Removes last ,
            table_fields.pop();
        }

        if !table_constrains.is_empty() {
            //Removes last ,
            table_constrains.pop();
        }

        let query = format!(
            "CREATE TABLE {} (\r\n{}\r\n{})",
            self.table_name, table_fields, table_constrains
        );
        #[no_context]
        sqlx::query(&query)
            .execute(exec)
            .await
            .with_context(context!(
                "table_name: {:?} | query: {:?}",
                self.table_name,
                query
            ))?;

        Ok(())
    }
}
