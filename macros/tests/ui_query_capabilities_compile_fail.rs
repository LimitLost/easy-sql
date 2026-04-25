/// Construct a fresh trybuild runner per test so a single failing fixture does not hide the rest.
fn compile_fail_fixture(path: &str) {
    trybuild::TestCases::new().compile_fail(path);
}

#[test]
fn unsupported_offset_capability_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/unsupported_offset_capability.rs");
}

#[test]
fn unsupported_exists_offset_capability_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/unsupported_exists_offset_capability.rs");
}

#[test]
fn unsupported_select_lock_sqlite_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/unsupported_select_lock_sqlite.rs");
}

#[test]
fn invalid_select_lock_placement_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_select_lock_placement.rs");
}

#[test]
fn invalid_select_lock_mode_token_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_select_lock_mode_token.rs");
}

#[test]
fn invalid_select_offset_without_limit_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_select_offset_without_limit.rs");
}

#[test]
fn invalid_exists_offset_without_limit_compile_fail() {
    if !cfg!(feature = "sqlite") {
        return;
    }

    compile_fail_fixture("tests/ui/invalid_exists_offset_without_limit.rs");
}