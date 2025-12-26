// This file contains tests that should FAIL to compile
// to verify that the compile-time checks in custom select are working.
//
// To test these, temporarily uncomment one at a time and verify compilation fails.

use crate::Table;

#[derive(Table)]
#[sql(version = 1)]
#[sql(unique_id = "e2f5f221-008c-4ffa-9454-fe0de3ad36ba")]
struct CompileFailTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    age: i64,
}

// COMPILE FAIL TEST 1: Using non-existent column in custom select
// Uncomment to verify this fails compilation with error about missing field
/*
#[derive(Output)]
#[sql(table = CompileFailTable)]
struct InvalidColumn {
    id: i32,
    #[sql(select = nonexistent_column || {arg0})]
    name: String,
}
*/

// COMPILE FAIL TEST 2: Using column from wrong table
// Uncomment to verify this fails compilation
/*
#[derive(Table)]
#[sql(version = 1)]
struct OtherTable {
    #[sql(primary_key)]
    id: i32,
    other_field: String,
}

#[derive(Output)]
#[sql(table = CompileFailTable)]
struct WrongTableColumn {
    id: i32,
    #[sql(select = OtherTable.other_field)]
    name: String,
}
*/

// COMPILE FAIL TEST 3: Using column with wrong type operation
// Uncomment to verify this fails compilation
/*
#[derive(Output)]
#[sql(table = CompileFailTable)]
struct WrongTypeOperation {
    id: i32,
    #[sql(select = name + age)] // Can't add string + number
    name: String,
}
*/
