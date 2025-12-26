// Temporary test to verify compile-time validation works
// This should be commented out after verification

use crate::Table;

#[derive(Table)]
#[sql(version = 1)]
#[sql(unique_id = "06e6dabd-a01e-41f5-82c3-f8880a647eb4")]
struct ValidationTestTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    age: i64,
}

// This should fail to compile because 'nonexistent' is not a column in ValidationTestTable
/* #[derive(Output)]
#[sql(table = ValidationTestTable)]
struct TestInvalidColumn {
    id: i32,
    #[sql(select = ValidationTestTable.nonexistent || {arg0})]
    name: String,
}
 */
