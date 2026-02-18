use easy_sql_macros::Output;

struct DummyTable {
    id: i32,
}

#[derive(Output)]
#[sql(table = DummyTable)]
#[sql(drivres = Driver)]
struct BadOutputStructAttr {
    id: i32,
}

fn main() {}
