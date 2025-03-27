use easy_macros::macros::always_context;

use crate::{QueryData, Sql};

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
//Remove in derive
#[always_context]
impl crate::SqlOutput<ExampleTableStruct, crate::Row> for ExampleStruct {
    fn sql_to_query<'a>(sql: &'a Sql<'a>) -> anyhow::Result<QueryData<'a>> {
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
                name: "field1".to_owned(),
                alias: None,
            },
            crate::RequestedColumn {
                name: "field2".to_owned(),
                alias: None,
            },
            crate::RequestedColumn {
                name: "field3".to_owned(),
                alias: None,
            },
        ];

        sql.query_output(requested_columns)
    }
    //Remove in derive
    #[no_context]
    fn convert<'r>(data: crate::Row) -> anyhow::Result<Self> {
        use anyhow::Context;
        use easy_macros::helpers::context;

        Ok(Self {
            field1: <crate::Row as crate::SqlxRow>::try_get(&data, "field1").with_context(
                context!("Getting field `field1` with type String for struct ExampleStruct"),
            )?,
            field2: <crate::Row as crate::SqlxRow>::try_get(&data, "field2").with_context(
                context!("Getting field `field2` with type i32 for struct ExampleStruct"),
            )?,
            field3: <crate::Row as crate::SqlxRow>::try_get(&data, "field3").with_context(
                context!("Getting field `field3` with type i64 for struct ExampleStruct"),
            )?,
        })
    }
}
