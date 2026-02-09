# What is this?

[![Crates.io](https://img.shields.io/crates/v/easy-sql-compilation-data.svg)](https://crates.io/crates/easy-sql-compilation-data)

Shared data-model crate for [`easy-sql`](https://crates.io/crates/easy-sql) build-time schema metadata stored in `easy_sql.ron`.

It provides serializable structs such as `CompilationData`, `TableDataVersion`, and helpers to locate/load/save the file. With the `migrations` feature it can generate migration token streams used by `easy-sql`’s macros.
