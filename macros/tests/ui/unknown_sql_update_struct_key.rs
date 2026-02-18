use easy_sql_macros::Update;

struct DummyTable {
    id: i32,
}

#[derive(Update)]
#[sql(table = DummyTable)]
#[sql(drivres = Driver)]
struct BadUpdateStructAttr {
    id: i32,
}

fn main() {}
