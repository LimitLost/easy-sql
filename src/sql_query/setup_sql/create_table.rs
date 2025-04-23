use std::collections::HashMap;

use anyhow::Context;
use async_trait::async_trait;
use easy_macros::{helpers::context, macros::always_context};

use crate::SetupSql;

use crate::TableField;

#[derive(Debug)]
pub struct CreateTable {
    pub table_name: &'static str,
    pub fields: Vec<TableField>,

    pub primary_keys: Vec<&'static str>,
    ///Can only be used when with single primary key
    pub auto_increment: bool,
    ///Key - table name
    ///Value - field names, foreign field names, on delete/update cascade
    pub foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
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
        let mut primary_keys = Vec::new();

        for field in self.fields.into_iter() {
            let TableField {
                name,
                data_type,
                is_unique,
                is_not_null,
            } = field;

            let date_type_sqlite = data_type.sqlite();

            primary_keys.push(name.clone());
            let unique = if is_unique { "UNIQUE" } else { "" };
            let not_null = if is_not_null { "NOT NULL" } else { "" };

            table_fields.push_str(&format!(
                "{} {} {} {},",
                name, date_type_sqlite, unique, not_null
            ));
        }

        //Primary key constraint
        if self.auto_increment {
            match primary_keys.first() {
                Some(primary_key) if primary_keys.len() == 1 => {
                    table_constrains
                        .push_str(&format!("PRIMARY KEY ({} AUTOINCREMENT)", primary_key));
                }
                _ => anyhow::bail!("Auto increment is only supported for single primary key"),
            }
        } else {
            table_constrains.push_str(&format!("PRIMARY KEY ({})", primary_keys.join(", ")));
        }

        //Foreign key constraints
        for (foreign_table, (referenced_fields, foreign_fields, cascade)) in self.foreign_keys {
            let referenced_fields = referenced_fields.join(", ");
            let foreign_fields = foreign_fields.join(", ");
            let on_delete = if cascade { "ON DELETE CASCADE" } else { "" };
            let on_update = if cascade { "ON UPDATE CASCADE" } else { "" };
            table_constrains.push_str(&format!(
                "FOREIGN KEY ({}) REFERENCES {}({}) {} {},",
                referenced_fields, foreign_table, foreign_fields, on_delete, on_update
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
