use super::Sqlite;
use anyhow::Context;
use easy_macros::{always_context, context};

use super::table_field_definition;
use crate::{AlterTable, AlterTableSingle, EasyExecutor, traits::SetupSql};

#[always_context]
impl SetupSql<Sqlite> for AlterTable {
    type Output = ();

    async fn query(self, exec: &mut impl EasyExecutor<Sqlite>) -> anyhow::Result<Self::Output> {
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
                        .execute(exec.executor())
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
                    let column_def = table_field_definition(column);
                    let column_def = column_def.trim_end_matches(',').trim_end();
                    let query =
                        format!("ALTER TABLE {} ADD COLUMN {}", self.table_name, column_def);

                    let sqlx_query = sqlx::query(&query);

                    #[no_context]
                    sqlx_query
                        .execute(exec.executor())
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
                        .execute(exec.executor())
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
