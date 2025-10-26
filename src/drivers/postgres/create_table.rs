use std::collections::HashMap;
use std::ops::DerefMut;

use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::PgConnection;

use super::{Postgres, table_field_definition};
use crate::SetupSql;

use crate::TableField;

#[derive(Debug)]
pub struct CreateTable {
    pub table_name: &'static str,
    pub fields: Vec<TableField>,

    pub primary_keys: Vec<&'static str>,
    ///Key - table name
    ///Value - field names, foreign field names, on delete/update cascade
    pub foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
}

#[always_context]
impl SetupSql<Postgres> for CreateTable {
    type Output = ();

    async fn query(
        self,
        exec: &mut (impl DerefMut<Target = PgConnection> + Send + Sync),
    ) -> anyhow::Result<Self::Output> {
        let mut table_fields = String::new();
        let mut table_constrains = String::new();

        for field in self.fields.into_iter() {
            table_fields.push_str(&table_field_definition(field));
        }

        let primary_keys = self.primary_keys;
        //Primary key constraint
        // Format primary keys with proper quoting
        let formatted_keys: Vec<String> = primary_keys
            .iter()
            .map(|key| format!("\"{}\"", key))
            .collect();
        table_constrains.push_str(&format!("PRIMARY KEY ({}),", formatted_keys.join(", ")));

        //Foreign key constraints
        for (foreign_table, (referenced_fields, foreign_fields, cascade)) in self.foreign_keys {
            let referenced_fields: Vec<String> = referenced_fields
                .iter()
                .map(|field| format!("\"{}\"", field))
                .collect();
            let foreign_fields: Vec<String> = foreign_fields
                .iter()
                .map(|field| format!("\"{}\"", field))
                .collect();
            let referenced_fields = referenced_fields.join(", ");
            let foreign_fields = foreign_fields.join(", ");
            let on_delete = if cascade { "ON DELETE CASCADE" } else { "" };
            let on_update = if cascade { "ON UPDATE CASCADE" } else { "" };
            table_constrains.push_str(&format!(
                "FOREIGN KEY ({referenced_fields}) REFERENCES \"{foreign_table}\"({foreign_fields}) {on_delete} {on_update},"
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
            "CREATE TABLE \"{}\" (\r\n{}\r\n{})",
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
