use easy_macros::macros::always_context;
use sql_compilation_data::SqlType;

use super::SqlValueMaybeRef;

#[derive(Debug)]
pub struct TableField {
    pub name: String,
    pub data_type: SqlType,
    pub is_unique: bool,
    pub is_not_null: bool,
    pub default: Option<&'static SqlValueMaybeRef<'static>>,
}

#[always_context]
impl TableField {
    pub fn definition<'a>(self, bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>) -> String {
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
            bindings_list.push(default);
            "DEFAULT ?"
        } else {
            ""
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
