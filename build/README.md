# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-sql-build.svg)](https://crates.io/crates/easy-sql-build)

Build-time helper crate that loads and validates schema metadata from `easy_sql.ron` and prepares compilation data for macro expansion.

It builds on [`easy-sql-compilation-data`](https://crates.io/crates/easy-sql-compilation-data) and can optionally enable `migrations` and `check_duplicate_table_names` to drive macro-time checks and migration token generation used by [`easy-sql-macros`](https://crates.io/crates/easy-sql-macros).
