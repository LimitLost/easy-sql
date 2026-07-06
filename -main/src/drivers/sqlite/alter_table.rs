use super::Sqlite;
use anyhow::Context;
use easy_macros::{always_context, context};

use super::table_field_definition;
use crate::{
    EasyExecutor,
    driver::{AlterTable, AlterTableSingle},
    traits::SetupSql,
};

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
                AlterTableSingle::AddForeignKey {
                    columns,
                    referenced_table,
                    referenced_columns,
                    cascade,
                } => {
                    // SQLite can't `ALTER TABLE ADD CONSTRAINT`, so adding a foreign key needs a full table
                    // rebuild (the supported recipe). Delegated to a helper for readability.
                    #[no_context]
                    add_foreign_key_via_rebuild(
                        exec,
                        self.table_name,
                        &columns,
                        referenced_table,
                        &referenced_columns,
                        cascade,
                    )
                    .await
                    .with_context(context!(
                        "table_name: {:?} | add foreign key ({:?}) -> {}({:?}) | queries_before: {:?}",
                        self.table_name,
                        columns,
                        referenced_table,
                        referenced_columns,
                        queries_done
                    ))?;

                    queries_done.push(format!(
                        "REBUILD {} ADD FOREIGN KEY ({}) REFERENCES {}({})",
                        self.table_name,
                        columns.join(", "),
                        referenced_table,
                        referenced_columns.join(", ")
                    ));
                }
            }
        }

        Ok(())
    }
}

#[always_context(skip(!))]
#[no_context_inputs]
/// Adds a foreign-key constraint to an existing SQLite table via the documented table-rebuild recipe
/// (<https://sqlite.org/lang_altertable.html> §7): build a new table that carries the extra constraint, copy the
/// rows in, swap it into place, and restore the indexes/triggers.
/// Reason: SQLite has no `ALTER TABLE ADD CONSTRAINT`. The new table's column definitions are taken verbatim from
/// the existing table's `sqlite_master` DDL (so columns/defaults are preserved exactly) and only the new
/// `FOREIGN KEY` clause is injected. Data is copied into a *temporary-named* table, so the copy never appears under
/// the real table name to any change-watcher armed on the connection (the watcher filters by table name).
/// `PRAGMA foreign_key_check` fails the migration loudly if existing rows would violate the new constraint.
async fn add_foreign_key_via_rebuild(
    exec: &mut impl EasyExecutor<Sqlite>,
    table_name: &str,
    columns: &[&str],
    referenced_table: &str,
    referenced_columns: &[&str],
    cascade: bool,
) -> anyhow::Result<()> {
    // Step 1: read the table's current CREATE statement and its index/trigger DDLs (before any change).
    let old_ddl: String =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ?")
            .bind(table_name)
            .fetch_one(exec.executor())
            .await
            .with_context(|| format!("reading DDL for table `{table_name}`"))?;
    let aux_ddls: Vec<String> = sqlx::query_scalar(
        "SELECT sql FROM sqlite_master WHERE tbl_name = ? AND type IN ('index', 'trigger') AND sql IS NOT NULL",
    )
    .bind(table_name)
    .fetch_all(exec.executor())
    .await
    .with_context(|| format!("reading indexes/triggers for table `{table_name}`"))?;

    // Step 2: build the new table's body by injecting the foreign-key clause before the closing paren of the
    // existing definition. Taking the body from the first '(' discards the original `CREATE TABLE [IF NOT EXISTS]
    // name` header, so the temp name can be substituted cleanly.
    let on_actions = if cascade {
        " ON DELETE CASCADE ON UPDATE CASCADE"
    } else {
        ""
    };
    let fk_clause = format!(
        "FOREIGN KEY ({}) REFERENCES {}({}){}",
        columns.join(", "),
        referenced_table,
        referenced_columns.join(", "),
        on_actions
    );
    let body_start = old_ddl
        .find('(')
        .with_context(|| format!("table `{table_name}` DDL has no '(': {old_ddl:?}"))?;
    let body = &old_ddl[body_start..];
    let close = body
        .rfind(')')
        .with_context(|| format!("table `{table_name}` DDL has no ')': {old_ddl:?}"))?;
    let new_body = format!("{}, {}{}", &body[..close], fk_clause, &body[close..]);
    let tmp_name = format!("_easy_sql_fkrebuild_{table_name}");

    // Step 3: run the rebuild. `PRAGMA foreign_keys=OFF` must be set OUTSIDE a transaction (it is a no-op
    // inside one), then the swap runs in a transaction for atomicity.
    run(exec, "PRAGMA foreign_keys = OFF").await?;
    run(exec, "BEGIN").await?;
    run(exec, &format!("CREATE TABLE {tmp_name} {new_body}")).await?;
    // The copy targets the TEMP name, so a watcher armed on the connection never sees it as the real table.
    run(
        exec,
        &format!("INSERT INTO {tmp_name} SELECT * FROM {table_name}"),
    )
    .await?;
    run(exec, &format!("DROP TABLE {table_name}")).await?;
    run(exec, &format!("ALTER TABLE {tmp_name} RENAME TO {table_name}")).await?;
    // Recreate indexes/triggers the DROP removed (their DDL references the table by name, now restored).
    for ddl in &aux_ddls {
        run(exec, ddl).await?;
    }

    // Step 4: fail loudly if existing rows violate the new constraint (no silent data loss).
    let violations = sqlx::query(&format!("PRAGMA foreign_key_check({table_name})"))
        .fetch_all(exec.executor())
        .await
        .with_context(|| format!("foreign_key_check for `{table_name}`"))?;
    if !violations.is_empty() {
        run(exec, "ROLLBACK").await.ok();
        run(exec, "PRAGMA foreign_keys = ON").await.ok();
        anyhow::bail!(
            "adding foreign key {:?} -> {}({:?}) to `{}` failed: {} existing row(s) violate it (orphaned references). Resolve the orphaned rows and retry.",
            columns,
            referenced_table,
            referenced_columns,
            table_name,
            violations.len()
        );
    }

    run(exec, "COMMIT").await?;
    run(exec, "PRAGMA foreign_keys = ON").await?;
    Ok(())
}

#[always_context(skip(!))]
#[no_context_inputs]
/// Executes one raw statement on the executor, attaching the statement to any error.
async fn run(exec: &mut impl EasyExecutor<Sqlite>, query: &str) -> anyhow::Result<()> {
    sqlx::query(query)
        .execute(exec.executor())
        .await
        .with_context(|| format!("rebuild statement failed: {query}"))?;
    Ok(())
}
