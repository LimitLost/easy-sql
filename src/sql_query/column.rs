use easy_macros::macros::always_context;

use sql_compilation_data::SqlType;

pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub ty: SqlType,
}
#[derive(Debug)]
pub struct RequestedColumn {
    pub name: String,
    pub alias: Option<String>,
}

#[always_context]
impl RequestedColumn {
    pub fn to_query_data(&self) -> String {
        if let Some(alias) = &self.alias {
            format!("`{}` AS `{}`", self.name, alias)
        } else {
            format!("`{}`", self.name)
        }
    }
}
