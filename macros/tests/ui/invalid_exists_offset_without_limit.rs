use easy_sql::{Table, query};

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiExistsOffsetTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    value: i32,
}

fn main() {
    let dummy_conn =
        easy_sql::macro_support::never_any::<&mut easy_sql::driver::DriverConnection<easy_sql::Sqlite>>();

    let _ = query!(<easy_sql::Sqlite> dummy_conn,
        EXISTS UiExistsOffsetTable OFFSET 1
    );
}
