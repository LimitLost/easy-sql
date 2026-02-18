use easy_sql_macros::Update;

struct DummyTable {
    id: i32,
}

#[derive(Update)]
#[sql(table = DummyTable)]
struct BadUpdateFieldAttr {
    #[sql(bytse)]
    id: i32,
}

fn main() {}
