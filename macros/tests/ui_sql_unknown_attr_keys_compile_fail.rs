/// One test per fixture keeps trybuild failures isolated to the exact unsupported attribute key.
fn compile_fail_fixture(path: &str) {
    trybuild::TestCases::new().compile_fail(path);
}

#[test]
fn unknown_sql_struct_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_struct_key.rs");
}

#[test]
fn unknown_sql_field_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_field_key.rs");
}

#[test]
fn unknown_sql_insert_multiple_unknown_keys_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_insert_multiple_unknown_keys.rs");
}

#[test]
fn unknown_sql_update_struct_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_update_struct_key.rs");
}

#[test]
fn unknown_sql_update_field_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_update_field_key.rs");
}

#[test]
fn unknown_sql_output_struct_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_output_struct_key.rs");
}

#[test]
fn unknown_sql_output_field_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_output_field_key.rs");
}

#[test]
fn unknown_sql_table_struct_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_table_struct_key.rs");
}

#[test]
fn unknown_sql_table_struct_key_table_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_table_struct_key_table.rs");
}

#[test]
fn unknown_sql_table_struct_key_name_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_table_struct_key_name.rs");
}

#[test]
fn unknown_sql_table_field_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_table_field_key.rs");
}

#[test]
fn unknown_sql_table_field_default_duplicate_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_table_field_default_duplicate.rs");
}

#[test]
fn unknown_sql_table_field_default_invalid_expr_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_table_field_default_invalid_expr.rs");
}

#[test]
fn unknown_sql_database_setup_struct_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_database_setup_struct_key.rs");
}

#[test]
fn unknown_sql_database_setup_field_key_compile_fail() {
    compile_fail_fixture("tests/ui/unknown_sql_database_setup_field_key.rs");
}

