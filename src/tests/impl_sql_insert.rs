use anyhow::Context;
use easy_macros::macros::always_context;

use crate::{Connection, SqlInsert, SqlTable, SqlValueMaybeRef};

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
impl SqlInsert<ExampleTableStruct> for ExampleStruct {
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

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<SqlValueMaybeRef<'_>>>> {
        Ok(vec![vec![
            crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::String(&self.field0)),
            crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::String(&self.field1)),
            crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I32(&self.field2)),
            crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I64(&self.field3)),
            crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I16(&self.field4)),
        ]])
    }
}

#[always_context]
impl SqlTable for ExampleTableStruct {
    fn table_name() -> &'static str {
        "table"
    }
}

#[always_context]
async fn test(conn: &mut Connection) -> anyhow::Result<()> {
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
