use std::ops::DerefMut;

use super::Sqlite;
use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};
use sqlx::SqliteConnection;

use super::table_field_definition;
use crate::{SetupSql, TableField};

pub enum AlterTableSingle {
    RenameTable {
        new_table_name: &'static str,
    },
    AddColumn {
        column: TableField,
    },
    RenameColumn {
        old_column_name: &'static str,
        new_column_name: &'static str,
    },
}

pub struct AlterTable {
    pub table_name: &'static str,
    pub alters: Vec<AlterTableSingle>,
}

#[always_context]
impl SetupSql<Sqlite> for AlterTable {
    type Output = ();

    async fn query(
        self,
        exec: &mut (impl DerefMut<Target = SqliteConnection> + Send + Sync),
    ) -> anyhow::Result<Self::Output> {
        let mut queries_done = Vec::new();

        for alter in self.alters {
            match alter {
                AlterTableSingle::RenameTable { new_table_name } => {
                    let query = format!(
                        "ALTER TABLE {} RENAME TO {}",
                        self.table_name, new_table_name
                    );

                    #[no_context]
                    sqlx::query(&query)
                        .execute(exec.deref_mut())
                        .await
                        .with_context(context!(
                            "table_name: {:?} | query: {:?} | queries_before: {:?}",
                            self.table_name,
                            query,
                            queries_done
                        ))?;

                    queries_done.push(query);
                }
                AlterTableSingle::AddColumn { column } => {
                    let query = format!(
                        "ALTER TABLE {} ADD COLUMN {}",
                        self.table_name,
                        table_field_definition(column)
                    );

                    let sqlx_query = sqlx::query(&query);

                    #[no_context]
                    sqlx_query
                        .execute(exec.deref_mut())
                        .await
                        .with_context(context!(
                            "table_name: {:?} | query: {:?} | queries_before: {:?}",
                            self.table_name,
                            query,
                            queries_done
                        ))?;

                    queries_done.push(query);
                }
                AlterTableSingle::RenameColumn {
                    old_column_name,
                    new_column_name,
                } => {
                    let query = format!(
                        "ALTER TABLE {} RENAME COLUMN {} TO {}",
                        self.table_name, old_column_name, new_column_name
                    );

                    #[no_context]
                    sqlx::query(&query)
                        .execute(exec.deref_mut())
                        .await
                        .with_context(context!(
                            "table_name: {:?} | query: {:?} | queries_before: {:?}",
                            self.table_name,
                            query,
                            queries_done
                        ))?;

                    queries_done.push(query);
                }
            }
        }

        Ok(())
    }
}
