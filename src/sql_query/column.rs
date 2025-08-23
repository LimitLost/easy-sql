use easy_macros::macros::always_context;

use sql_compilation_data::SqlType;

pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub ty: SqlType,
}
#[derive(Debug)]
pub struct RequestedColumn {
    pub table_name: Option<&'static str>,
    pub name: String,
    pub alias: Option<String>,
}

#[always_context]
impl RequestedColumn {
    pub fn to_query_data(&self) -> String {
        let table_name = if let Some(table) = self.table_name {
            format!("`{table}`.")
        } else {
            String::new()
        };
        if let Some(alias) = &self.alias {
            format!("{table_name}`{}` AS `{alias}`", self.name)
        } else {
            format!("{table_name}`{}`", self.name)
        }
    }
}
