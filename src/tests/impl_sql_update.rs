use easy_macros::macros::always_context;

use crate::SqlUpdate;

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
impl SqlUpdate<ExampleTableStruct> for ExampleStruct {
    fn updates(&self) -> anyhow::Result<Vec<(String, crate::SqlValueMaybeRef<'_>)>> {
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
                crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::String(&self.field1)),
            ),
            (
                "field2".to_string(),
                crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I32(&self.field2)),
            ),
            (
                "field3".to_string(),
                crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I64(&self.field3)),
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
impl SqlUpdate<ExampleTableStruct> for ExampleStruct2 {
    fn updates(&self) -> anyhow::Result<Vec<(String, crate::SqlValueMaybeRef<'_>)>> {
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
            crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::String(&self.field1)),
        ));
        if let Some(field2) = &self.field2 {
            updates.push((
                "field2".to_string(),
                crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I32(field2)),
            ));
        }
        if let Some(field3) = &self.field3 {
            updates.push((
                "field3".to_string(),
                crate::SqlValueMaybeRef::Ref(crate::SqlValueRef::I64(field3)),
            ));
        }
        Ok(updates)
    }
}
