use easy_sql::Table;

#[derive(Table)]
#[sql(no_version)]
#[sql(version = 1)]
#[sql(drivers = easy_sql::Sqlite)]
struct UiInvalidMigrationModeNoVersionAndVersion {
    #[sql(primary_key)]
    id: i64,
}

fn main() {}
