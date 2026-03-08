use easy_sql_macros::Table;

#[derive(Table)]
#[sql(no_version)]
#[sql(drivers = Driver)]
struct BadDefaultExpr {
    #[sql(primary_key)]
    id: i32,
    #[sql(default = 1 +)]
    value: i32,
}

fn main() {}
