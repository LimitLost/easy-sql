use super::sql_type::SqlType;

pub struct TableField {
    pub name: String,
    pub data_type: SqlType,
    pub is_primary_key: bool,
    pub foreign_key: Option<ForeignKey>,
    pub is_unique: bool,
    pub is_not_null: bool,
}


pub struct ForeignKey{
    pub table_name: &'static str,
    pub referenced_field: &'static str,
}