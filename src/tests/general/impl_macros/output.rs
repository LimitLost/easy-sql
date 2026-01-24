use easy_macros::always_context;
use sqlx::ColumnIndex;

use crate::{Driver, DriverRow, InternalDriver, ToConvert};

use super::TestDriver;

#[allow(dead_code)]
struct ExampleTableStruct {
    field0: String,
    field1: String,
    field2: i32,
    field3: i64,
    field4: i16,
}

#[allow(dead_code)]
struct ExampleStruct {
    field1: String,
    field2: i32,
    field3: i64,
}
//Remove in derive
#[always_context]
impl<D: Driver> crate::Output<ExampleTableStruct, D> for ExampleStruct
where
    DriverRow<D>: ToConvert<D>,
    str: ColumnIndex<DriverRow<D>>,
    for<'x> String: sqlx::Decode<'x, InternalDriver<D>>,
    String: sqlx::Type<InternalDriver<D>>,
    for<'x> i32: sqlx::Decode<'x, InternalDriver<D>>,
    i32: sqlx::Type<InternalDriver<D>>,
    for<'x> i64: sqlx::Decode<'x, InternalDriver<D>>,
    i64: sqlx::Type<InternalDriver<D>>,
{
    type UsedForChecks = Self;
    type DataToConvert = DriverRow<D>;

    fn select(current_query: &mut String) {
        current_query.push_str("field1, field2, field3");
    }
    //Remove in derive
    #[no_context]
    fn convert(data: DriverRow<D>) -> anyhow::Result<Self> {
        use anyhow::Context;
        use easy_macros::context;

        let _ = |data: DriverRow<TestDriver>| {
            anyhow::Result::<Self>::Ok(Self {
                field1: <DriverRow<TestDriver> as crate::SqlxRow>::try_get(&data, "field1")
                    .with_context(context!(
                        "Getting field `field1` with type String for struct ExampleStruct"
                    ))?,
                field2: <DriverRow<TestDriver> as crate::SqlxRow>::try_get(&data, "field2")
                    .with_context(context!(
                        "Getting field `field2` with type i32 for struct ExampleStruct"
                    ))?,
                field3: <DriverRow<TestDriver> as crate::SqlxRow>::try_get(&data, "field3")
                    .with_context(context!(
                        "Getting field `field3` with type i64 for struct ExampleStruct"
                    ))?,
            })
        };

        Ok(Self {
            field1: <DriverRow<D> as crate::SqlxRow>::try_get(&data, "field1").with_context(
                context!("Getting field `field1` with type String for struct ExampleStruct"),
            )?,
            field2: <DriverRow<D> as crate::SqlxRow>::try_get(&data, "field2").with_context(
                context!("Getting field `field2` with type i32 for struct ExampleStruct"),
            )?,
            field3: <DriverRow<D> as crate::SqlxRow>::try_get(&data, "field3").with_context(
                context!("Getting field `field3` with type i64 for struct ExampleStruct"),
            )?,
        })
    }
}
