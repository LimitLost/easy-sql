use easy_sql_macros::Table;

#[derive(Table)]
#[sql(no_version)]
#[sql(drivers = Driver)]
struct BadDuplicateDefault {
    #[sql(primary_key)]
    id: i32,
    #[sql(default = 1)]
    #[sql(default = 2)]
    value: i32,
}

fn main() {}
