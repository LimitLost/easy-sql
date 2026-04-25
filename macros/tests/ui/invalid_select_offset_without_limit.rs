use easy_sql::{Output, Table, query_lazy};
#[allow(unused_imports)]
use easy_sql::macro_support::StreamExt;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiOffsetTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    value: i32,
}

#[derive(Output)]
#[sql(table = UiOffsetTable)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiOffsetOutput {
    value: i32,
}

fn main() {
    let _ = query_lazy!(<easy_sql::Sqlite>
        SELECT UiOffsetOutput FROM UiOffsetTable OFFSET 1
    );
}
