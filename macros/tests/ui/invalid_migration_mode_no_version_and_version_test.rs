use easy_sql::Table;

#[derive(Table)]
#[sql(no_version)]
#[sql(version_test = 1)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiInvalidMigrationModeNoVersionAndVersionTest {
    #[sql(primary_key)]
    id: i64,
}

fn main() {}
