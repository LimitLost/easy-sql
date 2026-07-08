/// Keep migration attribute failures isolated so the broken attribute is obvious from the test name.
fn compile_fail_fixture(path: &str) {
    trybuild::TestCases::new().compile_fail(path);
}

#[test]
fn invalid_migration_mode_no_version_and_version_compile_fail() {
    if !cfg!(feature = "_ui_tests") || !cfg!(feature = "migrations") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_migration_mode_no_version_and_version.rs");
}

#[test]
fn invalid_migration_mode_no_version_and_version_test_compile_fail() {
    if !cfg!(feature = "_ui_tests") || !cfg!(feature = "migrations") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_migration_mode_no_version_and_version_test.rs");
}

#[test]
fn invalid_migration_mode_version_and_version_test_compile_fail() {
    if !cfg!(feature = "_ui_tests") || !cfg!(feature = "migrations") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_migration_mode_version_and_version_test.rs");
}

#[test]
fn invalid_migration_mode_required_compile_fail() {
    if !cfg!(feature = "_ui_tests") || !cfg!(feature = "migrations") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_migration_mode_required.rs");
}