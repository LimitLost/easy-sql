use easy_sql_macros::Insert;

struct DummyTable {
    id: i32,
}

#[derive(Insert)]
#[sql(table = DummyTable)]
struct BadInsertFieldAttr {
    #[sql(bytse)]
    id: i32,
}

fn main() {}
