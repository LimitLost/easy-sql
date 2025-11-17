use easy_macros::always_context;

use crate::{DriverRow, QueryBuilder, QueryData, Sql};

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
impl crate::Output<ExampleTableStruct, TestDriver> for ExampleStruct {
    type DataToConvert = DriverRow<TestDriver>;
    fn sql_to_query<'a>(
        sql: Sql,
        builder: QueryBuilder<'a, TestDriver>,
    ) -> anyhow::Result<QueryData<'a, TestDriver>> {
        crate::never::never_fn(|| {
            //Check for validity
            let table_instance = crate::never::never_any::<ExampleTableStruct>();

            Self {
                field1: table_instance.field1,
                field2: table_instance.field2,
                field3: table_instance.field3,
            }
        });

        let requested_columns = vec![
            crate::RequestedColumn {
                table_name: None,
                name: "field1".to_owned(),
                alias: None,
            },
            crate::RequestedColumn {
                table_name: None,
                name: "field2".to_owned(),
                alias: None,
            },
            crate::RequestedColumn {
                table_name: None,
                name: "field3".to_owned(),
                alias: None,
            },
        ];

        sql.query_output(builder, requested_columns)
    }

    fn select_sqlx(current_query: &mut String) {
        current_query.push_str("field1, field2, field3");
    }
    //Remove in derive
    #[no_context]
    fn convert(data: DriverRow<TestDriver>) -> anyhow::Result<Self> {
        use anyhow::Context;
        use easy_macros::context;

        Ok(Self {
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
    }
}
