use std::collections::HashMap;
use std::ops::DerefMut;

use anyhow::Context;
use easy_macros::{always_context, context};
use sqlx::SqliteConnection;

use super::{Sqlite, table_field_definition};
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
impl SetupSql<Sqlite> for CreateTable {
    type Output = ();

    async fn query(
        self,
        exec: &mut (impl DerefMut<Target = SqliteConnection> + Send + Sync),
    ) -> anyhow::Result<Self::Output> {
        let mut table_fields = String::new();
        let mut table_constrains = String::new();

        for field in self.fields.into_iter() {
            table_fields.push_str(&table_field_definition(field));
        }

        let primary_keys = self.primary_keys;
        //Primary key constraint
        if self.auto_increment {
            match primary_keys.first() {
                Some(primary_key) if primary_keys.len() == 1 => {
                    table_constrains
                        .push_str(&format!("PRIMARY KEY ({primary_key} AUTOINCREMENT),"));
                }
                _ => anyhow::bail!("Auto increment is only supported for single primary key"),
            }
        } else {
            table_constrains.push_str(&format!("PRIMARY KEY ({}),", primary_keys.join(", ")));
        }

        //Foreign key constraints
        for (foreign_table, (referenced_fields, foreign_fields, cascade)) in self.foreign_keys {
            let referenced_fields = referenced_fields.join(", ");
            let foreign_fields = foreign_fields.join(", ");
            let on_delete = if cascade { "ON DELETE CASCADE" } else { "" };
            let on_update = if cascade { "ON UPDATE CASCADE" } else { "" };
            table_constrains.push_str(&format!(
                "FOREIGN KEY ({referenced_fields}) REFERENCES {foreign_table}({foreign_fields}) {on_delete} {on_update},"
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

        let sqlx_query = sqlx::query(&query);

        #[no_context]
        sqlx_query
            .execute(exec.deref_mut())
            .await
            .with_context(context!(
                "table_name: {:?} | query: {:?}",
                self.table_name,
                query
            ))?;

        Ok(())
    }
}
