use easy_sql_macros::Table;

#[derive(Table)]
struct BadTableFieldAttr {
    #[sql(primray_key)]
    id: i32,
}

fn main() {}
