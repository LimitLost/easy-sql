#[derive(SqlInsert)]
#[sql(table = EasySqlTables)]
pub struct EasySqlTables{
    pub table_id: String,
    pub version: i64,
}

#[always_context]
impl EasySqlTables{
    pub async fn create(conn: &mut (impl EasyExecutor + Send + Sync), table_id: String, version: i64) -> anyhow::Result<()>{
        EasySqlTables::insert(conn, &EasySqlTables{table_id, version}).await?;

        Ok(())
    }

    pub async fn update_version(conn: &mut (impl EasyExecutor + Send + Sync), table_id: &str, new_version: i64) -> anyhow::Result<()> {
        EasySqlTables::update(conn,EasySqlTableVersion{version},sql_where!(table_id = {table_id})).await?;

        Ok(())
    }

    pub async fn get_version(conn: &mut (impl EasyExecutor + Send + Sync), table_id: &str) -> anyhow::Result<i64> {
        let version = EasySqlTables::select(conn, sql_where!(table_id = {table_id})).await?;
        Ok(version)
    }
}

#[derive(SqlUpdate,SqlOutput)]
#[sql(table = EasySqlTables)]
struct EasySqlTableVersion{
    pub version:i64
}

#[always_context]
impl SqlTable for EasySqlTables{
    fn table_name()->&'static str{
        "easy_sql_tables"
    }
}

