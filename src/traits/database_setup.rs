use easy_macros::macros::always_context;

#[always_context]
pub trait DatabaseSetup {
    fn setup(used_table_names: &mut Vec<String>) -> anyhow::Result<()>;
}
