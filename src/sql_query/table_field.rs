use easy_macros::macros::always_context;
use sql_compilation_data::SqlType;

#[derive(Debug)]
pub struct TableField {
    pub name: String,
    pub data_type: SqlType,
    pub is_unique: bool,
    pub is_not_null: bool,
    pub default: Option<String>,
}

#[always_context]
impl TableField {
    pub fn definition(self) -> String {
        let TableField {
            name,
            data_type,
            is_unique,
            is_not_null,
            default,
        } = self;

        let date_type_sqlite = data_type.sqlite();

        let unique = if is_unique { "UNIQUE" } else { "" };
        let not_null = if is_not_null { "NOT NULL" } else { "" };
        let default = if let Some(default) = default {
            format!("DEFAULT {}", default)
        } else {
            "".to_string()
        };

        format!(
            "{} {} {} {} {},",
            name, date_type_sqlite, unique, not_null, default
        )
    }
}

#[derive(Debug)]
pub struct ForeignKey {
    pub table_name: &'static str,
    pub referenced_field: &'static str,
}
