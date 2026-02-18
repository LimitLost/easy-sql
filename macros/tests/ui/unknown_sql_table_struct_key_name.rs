use easy_sql_macros::Table;

#[derive(Table)]
#[sql(name = "users")]
struct BadTableStructAttrName {
    #[sql(primary_key)]
    id: i32,
}

fn main() {}
