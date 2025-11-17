use anyhow::Context;
use easy_macros::always_context;

use super::{DatabaseInternalDefault, TestDriver};

use crate::{Connection, DriverArguments, Insert, QueryBuilder, Table, TableJoin};
#[allow(dead_code)]
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
#[no_context]
impl<'a> Insert<'a, ExampleTableStruct, TestDriver> for ExampleStruct {
    fn insert_columns() -> Vec<String> {
        let _ = || {
            //Check for validity
            let this_instance = crate::macro_support::never_any::<Self>();

            ExampleTableStruct {
                id: Default::default(),
                field0: this_instance.field0,
                field1: this_instance.field1,
                field2: this_instance.field2,
                field3: this_instance.field3,
                field4: this_instance.field4,
            }
        };
        vec![
            "field0".to_string(),
            "field1".to_string(),
            "field2".to_string(),
            "field3".to_string(),
            "field4".to_string(),
        ]
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, TestDriver>) -> anyhow::Result<usize> {
        // Fully safe because we pass by value, not by reference
        unsafe {
            builder
                .bind(self.field0)
                .context("Binding `field0` failed")?;
            builder.bind(self.field1)?;
            builder.bind(self.field2)?;
            builder.bind(self.field3)?;
            builder.bind(self.field4)?;
        }
        Ok(1)
    }

    fn insert_values_sqlx(
        self,
        args_list: DriverArguments<'a, TestDriver>,
    ) -> anyhow::Result<(DriverArguments<'a, TestDriver>, usize)> {
        let mut args = args_list;
        use sqlx::Arguments;
        args.add(self.field0)
            .map_err(anyhow::Error::from_boxed)
            .context("Failed to add `field0` to the sqlx arguments list")?;
        args.add(self.field1).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field2).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field3).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field4).map_err(anyhow::Error::from_boxed)?;
        Ok((args, 1))
    }
}

#[always_context]
impl<'a> Insert<'a, ExampleTableStruct, TestDriver> for &'a ExampleStruct {
    fn insert_columns() -> Vec<String> {
        //Validity check is only done in implementation for owned type
        /* crate::never::never_fn(|| {
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
        }); */
        vec![
            "field0".to_string(),
            "field1".to_string(),
            "field2".to_string(),
            "field3".to_string(),
            "field4".to_string(),
        ]
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, TestDriver>) -> anyhow::Result<usize> {
        unsafe {
            builder
                .bind(&self.field0)
                .with_context(|| format!("Binding `field0` (= {:?}) failed", &self.field0))?;
            builder.bind(&self.field1)?;
            builder.bind(&self.field2)?;
            builder.bind(&self.field3)?;
            builder.bind(&self.field4)?;
        }
        Ok(1)
    }

    fn insert_values_sqlx(
        self,
        mut args_list: DriverArguments<'a, TestDriver>,
    ) -> anyhow::Result<(DriverArguments<'a, TestDriver>, usize)> {
        use sqlx::Arguments;
        args_list
            .add(&self.field0)
            .map_err(anyhow::Error::from_boxed)
            .with_context(|| {
                format!(
                    "Failed to add `field0` to the sqlx arguments list | `field0` = {:?}",
                    &self.field0
                )
            })?;
        args_list
            .add(&self.field1)
            .map_err(anyhow::Error::from_boxed)?;
        args_list
            .add(&self.field2)
            .map_err(anyhow::Error::from_boxed)?;
        args_list
            .add(&self.field3)
            .map_err(anyhow::Error::from_boxed)?;
        args_list
            .add(&self.field4)
            .map_err(anyhow::Error::from_boxed)?;
        Ok((args_list, 1))
    }
}

#[always_context]
impl Table<TestDriver> for ExampleTableStruct {
    fn table_name() -> &'static str {
        "table"
    }
    fn primary_keys() -> Vec<&'static str> {
        vec!["id"]
    }

    fn table_joins(_builder: &mut QueryBuilder<'_, TestDriver>) -> Vec<TableJoin> {
        vec![]
    }
}

#[always_context]
/// Test used just for compile time checking of the SqlInsert macro implementation
async fn _test(conn: &mut Connection<TestDriver, DatabaseInternalDefault>) -> anyhow::Result<()> {
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
