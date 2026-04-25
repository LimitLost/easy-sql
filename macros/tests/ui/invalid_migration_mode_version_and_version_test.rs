use easy_sql::Table;

#[derive(Table)]
#[sql(version = 1)]
#[sql(version_test = 1)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiInvalidMigrationModeVersionAndVersionTest {
    #[sql(primary_key)]
    id: i64,
}

fn main() {}
