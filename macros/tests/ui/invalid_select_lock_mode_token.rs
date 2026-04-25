use easy_sql::{Output, Table, query_lazy};
#[allow(unused_imports)]
use easy_sql::macro_support::StreamExt;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiLockModeTokenTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    value: i32,
}

#[derive(Output)]
#[sql(table = UiLockModeTokenTable)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiLockModeTokenOutput {
    value: i32,
}

fn main() {
    let _ = query_lazy!(<easy_sql::Sqlite>
        SELECT UiLockModeTokenOutput FROM UiLockModeTokenTable FOR INVALID
    );
}
