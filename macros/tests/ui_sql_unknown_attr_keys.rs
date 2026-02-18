#[test]
fn sql_unknown_attr_keys_compile_fail() {
    let test_cases = trybuild::TestCases::new();
    test_cases.compile_fail("tests/ui/unknown_sql_struct_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_field_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_insert_multiple_unknown_keys.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_update_struct_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_update_field_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_output_struct_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_output_field_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_table_struct_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_table_struct_key_table.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_table_struct_key_name.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_table_field_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_database_setup_struct_key.rs");
    test_cases.compile_fail("tests/ui/unknown_sql_database_setup_field_key.rs");
}
