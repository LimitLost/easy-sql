use sql_compilation_data::SqlType;

use crate::Driver;

#[derive(Debug)]
pub struct TableField<'a, D: Driver> {
    pub name: &'static str,
    pub data_type: SqlType,
    pub is_unique: bool,
    pub is_not_null: bool,
    pub default: Option<&'a D::Value<'a>>,
    pub is_auto_increment: bool,
}

// Driver-specific implementations are in each driver module

#[derive(Debug)]
pub struct ForeignKey {
    pub table_name: &'static str,
    pub referenced_field: &'static str,
}
