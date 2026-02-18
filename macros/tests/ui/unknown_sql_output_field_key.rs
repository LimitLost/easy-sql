use easy_sql_macros::Output;

struct DummyTable {
    id: i32,
}

#[derive(Output)]
#[sql(table = DummyTable)]
struct BadOutputFieldAttr {
    #[sql(fiedl = DummyTable.id)]
    id: i32,
}

fn main() {}
