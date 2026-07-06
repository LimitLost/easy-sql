use anyhow::Context;
use easy_macros::{always_context, context};

use super::{Postgres, table_field_definition};
use crate::{
    EasyExecutor,
    driver::{AlterTable, AlterTableSingle},
    traits::SetupSql,
};

#[always_context]
impl SetupSql<Postgres> for AlterTable {
    type Output = ();

    async fn query(self, exec: &mut impl EasyExecutor<Postgres>) -> anyhow::Result<Self::Output> {
        let mut queries_done = Vec::new();

        for alter in self.alters {
            match alter {
                AlterTableSingle::RenameTable { new_table_name } => {
                    let query = format!(
                        "ALTER TABLE \"{}\" RENAME TO \"{}\"",
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
                    let query = format!(
                        "ALTER TABLE \"{}\" ADD COLUMN {}",
                        self.table_name, column_def
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
                AlterTableSingle::RenameColumn {
                    old_column_name,
                    new_column_name,
                } => {
                    let query = format!(
                        "ALTER TABLE \"{}\" RENAME COLUMN \"{}\" TO \"{}\"",
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
                AlterTableSingle::AddForeignKey {
                    columns,
                    referenced_table,
                    referenced_columns,
                    cascade,
                } => {
                    // Postgres supports adding a foreign key in place — no table rebuild needed (unlike SQLite).
                    let on_actions = if cascade {
                        " ON DELETE CASCADE ON UPDATE CASCADE"
                    } else {
                        ""
                    };
                    let quoted_columns = columns
                        .iter()
                        .map(|column| format!("\"{column}\""))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let quoted_referenced = referenced_columns
                        .iter()
                        .map(|column| format!("\"{column}\""))
                        .collect::<Vec<_>>()
                        .join(", ");
                    // A deterministic constraint name keyed on the local columns keeps the migration idempotent-ish
                    // and the error messages legible.
                    let constraint = format!("fk_{}_{}", self.table_name, columns.join("_"));
                    let query = format!(
                        "ALTER TABLE \"{}\" ADD CONSTRAINT \"{}\" FOREIGN KEY ({}) REFERENCES \"{}\"({}){}",
                        self.table_name,
                        constraint,
                        quoted_columns,
                        referenced_table,
                        quoted_referenced,
                        on_actions
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
