use anyhow::Context;
use easy_macros::always_context;

use super::{NeverConnection, TestDriver};

use crate::{Driver, DriverArguments, Insert, InternalDriver, Table, query};
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
impl<'a, D: Driver> Insert<'a, ExampleTableStruct, D> for ExampleStruct
where
    for<'x> String: sqlx::Encode<'x, InternalDriver<D>>,
    String: sqlx::Type<InternalDriver<D>>,
    for<'x> i32: sqlx::Encode<'x, InternalDriver<D>>,
    i32: sqlx::Type<InternalDriver<D>>,
    for<'x> i64: sqlx::Encode<'x, InternalDriver<D>>,
    i64: sqlx::Type<InternalDriver<D>>,
    for<'x> i16: sqlx::Encode<'x, InternalDriver<D>>,
    i16: sqlx::Type<InternalDriver<D>>,
{
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

    fn insert_values(
        self,
        args_list: DriverArguments<'a, D>,
    ) -> anyhow::Result<(DriverArguments<'a, D>, usize)> {
        let mut args = args_list;
        use sqlx::Arguments;

        let _ = |mut args: DriverArguments<'a, TestDriver>| {
            let _self = crate::macro_support::never_any::<Self>();
            args.add(_self.field0)
                .map_err(anyhow::Error::from_boxed)
                .context("Failed to add `field0` to the sqlx arguments list")?;
            args.add(_self.field1).map_err(anyhow::Error::from_boxed)?;
            args.add(_self.field2).map_err(anyhow::Error::from_boxed)?;
            args.add(_self.field3).map_err(anyhow::Error::from_boxed)?;
            args.add(_self.field4).map_err(anyhow::Error::from_boxed)?;

            anyhow::Result::<()>::Ok(())
        };

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

    fn insert_values(
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

    fn table_joins(_current_query: &mut String) {}
}

#[always_context(skip(!))]
/// Test used just for compile time checking of the SqlInsert macro implementation
async fn _test(mut conn: &mut NeverConnection) -> anyhow::Result<()> {
    let to_insert = ExampleStruct {
        field0: "value0".to_string(),
        field1: "value1".to_string(),
        field2: 2,
        field3: 3,
        field4: 4,
    };
    query!(conn, INSERT INTO ExampleTableStruct VALUES {to_insert}).await?;
    Ok(())
}
