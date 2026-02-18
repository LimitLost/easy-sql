use easy_sql_macros::Insert;

struct DummyTable {
    id: i32,
}

#[derive(Insert)]
#[sql(table = DummyTable)]
#[sql(drivres = Driver)]
struct BadInsertMultipleUnknownKeys {
    #[sql(bytse)]
    id: i32,
}

fn main() {}
