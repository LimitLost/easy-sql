fn main() {
    easy_macros::macros::always_context_build::build(&[regex::Regex::new(r"readme\.rs").unwrap()]);
    sql_build::build(&[regex::Regex::new(r"example_all\.rs").unwrap()]);
}
