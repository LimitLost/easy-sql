use easy_sql::{Output, Table, query};

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiLockTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    value: i32,
}

#[derive(Output)]
#[sql(table = UiLockTable)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiLockOutput {
    value: i32,
}

fn main() {
    let dummy_conn =
        easy_sql::macro_support::never_any::<&mut easy_sql::driver::DriverConnection<easy_sql::Sqlite>>();

    let _ = query!(<easy_sql::Sqlite> dummy_conn,
        SELECT UiLockOutput FROM UiLockTable FOR UPDATE
    );

    let _ = query!(<easy_sql::Sqlite> dummy_conn,
        SELECT UiLockOutput FROM UiLockTable FOR NO KEY UPDATE
    );

    let _ = query!(<easy_sql::Sqlite> dummy_conn,
        SELECT UiLockOutput FROM UiLockTable FOR SHARE
    );

    let _ = query!(<easy_sql::Sqlite> dummy_conn,
        SELECT UiLockOutput FROM UiLockTable FOR KEY SHARE
    );
}
