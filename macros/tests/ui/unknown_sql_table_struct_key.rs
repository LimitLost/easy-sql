use easy_sql_macros::Table;

#[derive(Table)]
#[sql(drivres = Driver)]
struct BadTableStructAttr {
    #[sql(primary_key)]
    id: i32,
}

fn main() {}
