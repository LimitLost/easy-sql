use std::collections::HashMap;

use async_trait::async_trait;
use easy_sql::macro_support::{Executor, Query as DriverQuery, ToConvert};
use easy_sql::{Driver, EasyExecutor, Output, Table, query};

#[derive(Debug)]
struct UiNoOffsetDriver;

impl Driver for UiNoOffsetDriver {
    type InternalDriver = easy_sql::driver::InternalDriver<easy_sql::Sqlite>;

    fn identifier_delimiter() -> &'static str {
        "\""
    }

    fn parameter_placeholder(_index: usize) -> String {
        "?".to_string()
    }

    async fn table_exists(
        _conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        _name: &'static str,
    ) -> anyhow::Result<bool> {
        unreachable!("compile-fail fixture")
    }

    async fn create_table(
        _conn: &mut (impl EasyExecutor<Self> + Send + Sync),
        _table_name: &'static str,
        _fields: Vec<easy_sql::driver::TableField>,
        _primary_keys: Vec<&'static str>,
        _foreign_keys: HashMap<&'static str, (Vec<&'static str>, Vec<&'static str>, bool)>,
    ) -> anyhow::Result<()> {
        unreachable!("compile-fail fixture")
    }
}

#[async_trait]
impl ToConvert<UiNoOffsetDriver> for easy_sql::driver::DriverRow<UiNoOffsetDriver> {
    async fn get<'a>(
        exec: impl Executor<'_, Database = easy_sql::driver::InternalDriver<UiNoOffsetDriver>>,
        query: DriverQuery<
            'a,
            easy_sql::driver::InternalDriver<UiNoOffsetDriver>,
            easy_sql::driver::DriverArguments<'a, UiNoOffsetDriver>,
        >,
    ) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(exec.fetch_one(query).await?)
    }
}

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
#[sql(drivers = UiNoOffsetDriver, easy_sql::Sqlite)]
struct UiNoOffsetTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    value: i32,
}

#[derive(Output)]
#[sql(table = UiNoOffsetTable)]
#[sql(drivers = UiNoOffsetDriver, easy_sql::Sqlite)]
struct UiNoOffsetOutput {
    value: i32,
}

fn main() {
    let dummy_conn = easy_sql::macro_support::never_any::<
        &mut easy_sql::driver::DriverConnection<easy_sql::Sqlite>,
    >();

    let _ = query!(<UiNoOffsetDriver> dummy_conn,
        SELECT UiNoOffsetOutput FROM UiNoOffsetTable LIMIT 10 OFFSET 1
    );
}
