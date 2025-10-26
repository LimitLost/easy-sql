#[derive(Debug)]
pub struct TableField {
    pub name: &'static str,
    pub data_type: String,
    pub is_unique: bool,
    pub is_not_null: bool,
    pub default: Option<String>,
    pub is_auto_increment: bool,
}

// Driver-specific implementations are in each driver module

#[derive(Debug)]
pub struct ForeignKey {
    pub table_name: &'static str,
    pub referenced_field: &'static str,
}
