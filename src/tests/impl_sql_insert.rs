use anyhow::Context;
use easy_macros::macros::always_context;

use crate::{
    Connection, Driver, SqlInsert, SqlTable, Sqlite, TableJoin, sqlite::DatabaseInternalDefault,
};

struct ExampleTableStruct {
    id: i64,
    field0: String,
    field1: String,
    field2: i32,
    field3: i64,
    field4: i16,
}
#[derive(Debug)]
struct ExampleStruct {
    field0: String,
    field1: String,
    field2: i32,
    field3: i64,
    field4: i16,
}

#[always_context]
impl SqlInsert<ExampleTableStruct, Sqlite> for ExampleStruct {
    fn insert_columns() -> Vec<String> {
        crate::never::never_fn(|| {
            //Check for validity
            let this_instance = crate::never::never_any::<Self>();

            ExampleTableStruct {
                //TODO Check if default value is set (or is Option<> or auto increment id) Use then default value
                id: Default::default(),
                field0: this_instance.field0,
                field1: this_instance.field1,
                field2: this_instance.field2,
                field3: this_instance.field3,
                field4: this_instance.field4,
            }
        });
        vec![
            "field0".to_string(),
            "field1".to_string(),
            "field2".to_string(),
            "field3".to_string(),
            "field4".to_string(),
        ]
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<<Sqlite as Driver>::Value<'_>>>> {
        Ok(vec![vec![
            (&self.field0).into(),
            (&self.field1).into(),
            (&self.field2).into(),
            (&self.field3).into(),
            (&self.field4).into(),
        ]])
    }
}

#[always_context]
impl SqlTable<Sqlite> for ExampleTableStruct {
    fn table_name() -> &'static str {
        "table"
    }
    fn primary_keys() -> Vec<&'static str> {
        vec!["id"]
    }

    fn table_joins() -> Vec<TableJoin<'static, Sqlite>> {
        vec![]
    }
}

#[always_context]
async fn test(conn: &mut Connection<Sqlite, DatabaseInternalDefault>) -> anyhow::Result<()> {
    let to_insert = ExampleStruct {
        field0: "value0".to_string(),
        field1: "value1".to_string(),
        field2: 2,
        field3: 3,
        field4: 4,
    };
    ExampleTableStruct::insert(conn, &to_insert).await?;
    Ok(())
}
