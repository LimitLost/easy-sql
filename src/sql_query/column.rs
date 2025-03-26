use super::sql_type::SqlType;

pub struct Column {
    pub name: String,
    pub alias: Option<String>,
    pub ty: SqlType,
}

pub struct RequestedColumn {
    pub name: String,
    pub alias: Option<String>,
}
