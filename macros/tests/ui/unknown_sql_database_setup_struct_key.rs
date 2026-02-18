use easy_sql_macros::DatabaseSetup;

#[derive(DatabaseSetup)]
#[sql(drivres = Driver)]
struct BadDatabaseSetupStructAttr;

fn main() {}
