use super::sql_type::SqlType;

pub struct TableField {
    pub name: String,
    pub data_type: SqlType,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
    pub is_unique: bool,
    pub is_not_null: bool,
}
