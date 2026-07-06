use crate::driver::TableField;

/// Single alter-table operation used by the migration procedural macros.
///
/// This enum is **not** intended to be constructed manually in user code; the
/// [`Table`](macro@crate::Table) macro generates these values when compiling migrations.
///
/// # ⚠️ API instability
///
/// This type is marked as `#[non_exhaustive]` and **will be changed in the
/// future**. Prefer to rely on the macros instead of matching all variants.
#[non_exhaustive]
pub enum AlterTableSingle {
    /// Rename an existing table.
    RenameTable { new_table_name: &'static str },
    /// Add a new column definition to an existing table.
    AddColumn { column: TableField },
    /// Rename an existing column.
    RenameColumn {
        old_column_name: &'static str,
        new_column_name: &'static str,
    },
    /// Add a foreign-key constraint to an existing table.
    ///
    /// SQLite cannot `ALTER TABLE ADD CONSTRAINT`, so its driver implements this with a table rebuild (the
    /// supported 12-step recipe); Postgres uses `ALTER TABLE ... ADD CONSTRAINT` directly. Only *adding* a
    /// foreign key is supported — removing or redefining one is not (the migration generator rejects those).
    AddForeignKey {
        /// The local columns that make up the foreign key.
        columns: Vec<&'static str>,
        /// The referenced table's name.
        referenced_table: &'static str,
        /// The referenced table's primary-key columns (the targets of `columns`).
        referenced_columns: Vec<&'static str>,
        /// Whether the constraint cascades on delete/update (vs. the default no-action).
        cascade: bool,
    },
}

/// Collection of alter-table operations for a single table.
///
/// This struct is used internally by the migration procedural macros; it is
/// not considered a stable public API. The [`Table`](macro@crate::Table) macro generates these values
/// as part of migration compilation, so prefer using the macro instead of
/// constructing this directly.
pub struct AlterTable {
    pub table_name: &'static str,
    pub alters: Vec<AlterTableSingle>,
}
