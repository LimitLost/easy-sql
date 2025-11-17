fn main() {
    easy_macros::always_context_build::build(&[regex::Regex::new(r"readme\.rs").unwrap()]);
    sql_build::build(
        &[regex::Regex::new(r"example_all\.rs").unwrap()],
        #[cfg(not(any(feature = "sqlite", feature = "postgres")))]
        &[],
        #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
        &["crate::Sqlite"],
        #[cfg(all(feature = "postgres", not(feature = "sqlite")))]
        &["crate::Postgres"],
        #[cfg(all(feature = "sqlite", feature = "postgres"))]
        &["crate::Sqlite", "crate::Postgres"],
    );
}
