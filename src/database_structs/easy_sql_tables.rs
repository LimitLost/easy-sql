use easy_macros::macros::always_context;

use crate::SqlTable;

pub struct EasySqlTables {
    table_id: String,
    table_version: i64,
}

#[always_context]
impl SqlTable for EasySqlTables {
    fn table_name() -> &'static str {
        "easy_sql_tables"
    }
}
