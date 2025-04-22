use sql_compilation_data::SqlType;

#[derive(Debug)]
pub struct TableField {
    pub name: String,
    pub data_type: SqlType,
    pub is_unique: bool,
    pub is_not_null: bool,
}
#[derive(Debug)]
pub struct ForeignKey {
    pub table_name: &'static str,
    pub referenced_field: &'static str,
}
