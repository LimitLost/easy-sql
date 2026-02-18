use easy_sql_macros::Table;

#[derive(Table)]
#[sql(table = "users")]
struct BadTableStructAttrTable {
    #[sql(primary_key)]
    id: i32,
}

fn main() {}
