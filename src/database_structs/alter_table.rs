use crate::TableField;

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
