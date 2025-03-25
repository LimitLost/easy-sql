pub trait DatabaseSetup {
    fn setup(used_table_names: &mut Vec<String>) -> anyhow::Result<()>;
}
