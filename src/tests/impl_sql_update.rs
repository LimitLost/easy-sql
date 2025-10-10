use easy_macros::macros::always_context;

use crate::{SqlUpdate, Sqlite};

struct ExampleTableStruct {
    field0: String,
    field1: String,
    field2: i32,
    field3: i64,
    field4: i16,
}

struct ExampleStruct {
    field1: String,
    field2: i32,
    field3: i64,
}

#[always_context]
impl SqlUpdate<ExampleTableStruct, Sqlite> for ExampleStruct {
    fn updates(&mut self) -> anyhow::Result<Vec<(String, crate::SqlExpr<'_, Sqlite>)>> {
        crate::never::never_fn(|| {
            //Check for validity
            let update_instance = crate::never::never_any::<Self>();
            let mut table_instance = crate::never::never_any::<ExampleTableStruct>();

            table_instance.field1 = update_instance.field1;
            table_instance.field2 = update_instance.field2;
            table_instance.field3 = update_instance.field3;
        });
        Ok(vec![
            (
                "field1".to_string(),
                crate::SqlExpr::Value((&self.field1).into()),
            ),
            (
                "field2".to_string(),
                crate::SqlExpr::Value((&self.field2).into()),
            ),
            (
                "field3".to_string(),
                crate::SqlExpr::Value((&self.field3).into()),
            ),
        ])
    }
}

struct ExampleStruct2 {
    field1: String,
    field2: Option<i32>,
    field3: Option<i64>,
}

#[always_context]
impl SqlUpdate<ExampleTableStruct, Sqlite> for ExampleStruct2 {
    fn updates(&mut self) -> anyhow::Result<Vec<(String, crate::SqlExpr<'_, Sqlite>)>> {
        //If Option is set to None then ignore
        crate::never::never_fn(|| {
            //Check for validity
            let update_instance = crate::never::never_any::<Self>();
            let mut table_instance = crate::never::never_any::<ExampleTableStruct>();

            table_instance.field1 = update_instance.field1;
            table_instance.field2 = update_instance.field2.unwrap();
            table_instance.field3 = update_instance.field3.unwrap();
        });
        let mut updates = Vec::new();
        updates.push((
            "field1".to_string(),
            crate::SqlExpr::Value((&self.field1).into()),
        ));
        if let Some(field2) = &self.field2 {
            updates.push(("field2".to_string(), crate::SqlExpr::Value((field2).into())));
        }
        if let Some(field3) = &self.field3 {
            updates.push(("field3".to_string(), crate::SqlExpr::Value((field3).into())));
        }
        Ok(updates)
    }
}
