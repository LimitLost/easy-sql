use easy_sql::Table;

#[derive(Table)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiInvalidMigrationModeRequired {
    #[sql(primary_key)]
    id: i64,
}

fn main() {}
