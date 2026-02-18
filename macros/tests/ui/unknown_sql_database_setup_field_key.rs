use easy_sql_macros::DatabaseSetup;

#[derive(DatabaseSetup)]
struct BadDatabaseSetupFieldAttr {
    #[sql(bytes)]
    table: i32,
}

fn main() {}
