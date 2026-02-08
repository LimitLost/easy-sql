/// Column definition used by drivers to build tables and migrations.
///
/// This struct is used when calling [`Driver::create_table`](crate::Driver::create_table) and
/// within [`AlterTableSingle::AddColumn`](crate::driver::AlterTableSingle::AddColumn). Driver backends
/// map the fields to SQL column definitions appropriate for their dialect.
///
#[derive(Debug)]
pub struct TableField {
    /// Column name as it should appear in SQL.
    pub name: &'static str,
    /// Database-specific type name (e.g. `TEXT`, `INTEGER`, `UUID`).
    pub data_type: String,
    /// Whether to add a `UNIQUE` constraint.
    pub is_unique: bool,
    /// Whether to add a `NOT NULL` constraint.
    pub is_not_null: bool,
    /// Optional SQL literal used as `DEFAULT` value.
    pub default: Option<String>,
    /// Whether the column should auto-increment (driver-specific behavior).
    pub is_auto_increment: bool,
}
