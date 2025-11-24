//! Prototype output implementation for query! macro

use anyhow::Context;
use easy_macros::always_context;

use crate::{
    Connection, Driver, DriverArguments, DriverRow, Expr, Insert, Output, QueryBuilder, Table,
    TableJoin, Update, macro_support::never_any,
};

use super::{DatabaseInternalDefault, TestDriver};

struct ExampleTable {
    id: i64,
    field0: String,
    field1: String,
    field2: i32,
    field3: i64,
    field4: i16,
}

#[always_context]
impl Table<TestDriver> for ExampleTable {
    fn table_name() -> &'static str {
        "example_table"
    }

    fn primary_keys() -> Vec<&'static str> {
        vec!["id"]
    }

    fn table_joins(_builder: &mut QueryBuilder<'_, TestDriver>) -> Vec<TableJoin> {
        vec![]
    }
}

#[always_context]
#[no_context]
impl<'a> Insert<'a, ExampleTable, TestDriver> for ExampleTable {
    fn insert_columns() -> Vec<String> {
        vec![
            "id".into(),
            "field0".into(),
            "field1".into(),
            "field2".into(),
            "field3".into(),
            "field4".into(),
        ]
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, TestDriver>) -> anyhow::Result<usize> {
        unsafe {
            builder.bind(self.id)?;
            builder.bind(self.field0)?;
            builder.bind(self.field1)?;
            builder.bind(self.field2)?;
            builder.bind(self.field3)?;
            builder.bind(self.field4)?;
        }
        Ok(1)
    }

    fn insert_values_sqlx(
        self,
        mut args: DriverArguments<'a, TestDriver>,
    ) -> anyhow::Result<(DriverArguments<'a, TestDriver>, usize)> {
        use sqlx::Arguments;
        args.add(self.id).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field0).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field1).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field2).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field3).map_err(anyhow::Error::from_boxed)?;
        args.add(self.field4).map_err(anyhow::Error::from_boxed)?;
        Ok((args, 1))
    }
}

#[always_context]
impl<'a> Insert<'a, ExampleTable, TestDriver> for &'a ExampleTable {
    fn insert_columns() -> Vec<String> {
        ExampleTable::insert_columns()
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, TestDriver>) -> anyhow::Result<usize> {
        unsafe {
            builder.bind(&self.id)?;
            builder.bind(&self.field0)?;
            builder.bind(&self.field1)?;
            builder.bind(&self.field2)?;
            builder.bind(&self.field3)?;
            builder.bind(&self.field4)?;
        }
        Ok(1)
    }

    fn insert_values_sqlx(
        self,
        mut args: DriverArguments<'a, TestDriver>,
    ) -> anyhow::Result<(DriverArguments<'a, TestDriver>, usize)> {
        use sqlx::Arguments;
        args.add(&self.id).map_err(anyhow::Error::from_boxed)?;
        args.add(&self.field0).map_err(anyhow::Error::from_boxed)?;
        args.add(&self.field1).map_err(anyhow::Error::from_boxed)?;
        args.add(&self.field2).map_err(anyhow::Error::from_boxed)?;
        args.add(&self.field3).map_err(anyhow::Error::from_boxed)?;
        args.add(&self.field4).map_err(anyhow::Error::from_boxed)?;
        Ok((args, 1))
    }
}

#[always_context]
impl<'a> Update<'a, ExampleTable, TestDriver> for &'a ExampleTable {
    fn updates(
        self,
        builder: &mut QueryBuilder<'_, TestDriver>,
    ) -> anyhow::Result<Vec<(String, Expr)>> {
        unsafe {
            builder.bind(&self.id)?;
            builder.bind(&self.field0)?;
            builder.bind(&self.field1)?;
            builder.bind(&self.field2)?;
            builder.bind(&self.field3)?;
            builder.bind(&self.field4)?;
        }
        Ok(vec![
            ("id".to_string(), crate::Expr::Value),
            ("field0".to_string(), crate::Expr::Value),
            ("field1".to_string(), crate::Expr::Value),
            ("field2".to_string(), crate::Expr::Value),
            ("field3".to_string(), crate::Expr::Value),
            ("field4".to_string(), crate::Expr::Value),
        ])
    }

    fn updates_sqlx(
        self,
        mut args_list: DriverArguments<'a, TestDriver>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<DriverArguments<'a, TestDriver>> {
        use sqlx::Arguments;
        let _easy_sql_d = TestDriver::identifier_delimiter();

        args_list.add(self.id).map_err(anyhow::Error::from_boxed)?;
        current_query.push_str(&format!(
            "{_easy_sql_d}id{_easy_sql_d} = {}, ",
            TestDriver::parameter_placeholder(*parameter_n)
        ));
        *parameter_n += 1;

        args_list
            .add(&self.field0)
            .map_err(anyhow::Error::from_boxed)?;
        current_query.push_str(&format!(
            "{_easy_sql_d}field0{_easy_sql_d} = {}, ",
            TestDriver::parameter_placeholder(*parameter_n)
        ));
        *parameter_n += 1;

        args_list
            .add(&self.field1)
            .map_err(anyhow::Error::from_boxed)?;
        current_query.push_str(&format!(
            "{_easy_sql_d}field1{_easy_sql_d} = {}, ",
            TestDriver::parameter_placeholder(*parameter_n)
        ));
        *parameter_n += 1;

        args_list
            .add(&self.field2)
            .map_err(anyhow::Error::from_boxed)?;
        current_query.push_str(&format!(
            "{_easy_sql_d}field2{_easy_sql_d} = {}, ",
            TestDriver::parameter_placeholder(*parameter_n)
        ));
        *parameter_n += 1;

        args_list
            .add(&self.field3)
            .map_err(anyhow::Error::from_boxed)?;
        current_query.push_str(&format!(
            "{_easy_sql_d}field3{_easy_sql_d} = {}, ",
            TestDriver::parameter_placeholder(*parameter_n)
        ));
        *parameter_n += 1;

        args_list
            .add(&self.field4)
            .map_err(anyhow::Error::from_boxed)?;
        current_query.push_str(&format!(
            "{_easy_sql_d}field4{_easy_sql_d} = {}",
            TestDriver::parameter_placeholder(*parameter_n)
        ));
        *parameter_n += 1;

        Ok(args_list)
    }
}

#[allow(dead_code)]
struct ExampleOutput {
    field1: String,
    field2: i32,
    field3: i64,
}

#[always_context]
impl Output<ExampleTable, TestDriver> for ExampleOutput {
    type DataToConvert = DriverRow<TestDriver>;
    fn sql_to_query<'a>(
        sql: crate::Sql,
        builder: QueryBuilder<'a, TestDriver>,
    ) -> anyhow::Result<crate::QueryData<'a, TestDriver>> {
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

    #[no_context]
    fn convert<'r>(data: DriverRow<TestDriver>) -> anyhow::Result<Self> {
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

#[always_context]
#[no_context]
async fn _test_select() -> anyhow::Result<ExampleOutput> {
    let mut fake_conn = never_any::<Connection<TestDriver, DatabaseInternalDefault>>();

    let random_id = 42;

    // query!(&mut fake_conn, SELECT ExampleOutput FROM ExampleTable WHERE id = {random_id} AND field2 > 17);

    //TODO Add documentation for query! : "IMPORTANT: Await is called inside the macro and unfortunately cannot be moved outside of the invocation"

    // query! macro output
    let result = {
        //TODO Security checks
        // Imports
        use anyhow::Context;
        use {crate::ToConvert, sqlx::Arguments};
        let mut args = DriverArguments::<TestDriver>::default();
        let mut query = "SELECT ".to_string();
        let mut current_arg_n = 0;
        let mut _easy_sql_d = TestDriver::identifier_delimiter();
        // Build query
        ExampleOutput::select_sqlx(&mut query);
        query.push_str(" FROM ");
        query.push_str(ExampleTable::table_name());
        query.push_str(&format!(
            " WHERE {_easy_sql_d}id{_easy_sql_d} = {} AND {_easy_sql_d}field2{_easy_sql_d} > {}",
            TestDriver::parameter_placeholder(current_arg_n),
            TestDriver::parameter_placeholder(current_arg_n + 1)
        ));
        args.add(&random_id).map_err(anyhow::Error::from_boxed)?;
        current_arg_n += 1;

        let mut builder = sqlx::QueryBuilder::with_arguments(query, args);
        let built_query = builder.build();

        async fn execute<'a>(
            exec: impl sqlx::Executor<'a, Database = crate::InternalDriver<TestDriver>>,
            query: sqlx::query::Query<
                'a,
                crate::InternalDriver<TestDriver>,
                DriverArguments<'a, TestDriver>,
            >,
        ) -> anyhow::Result<ExampleOutput> {
            let raw_data = <ExampleOutput as Output<ExampleTable, TestDriver>>::DataToConvert::get(
                exec, query,
            )
            .await?;

            let result = <ExampleOutput as Output<ExampleTable, TestDriver>>::convert(raw_data)?;

            Ok(result)
        }

        execute(&mut *fake_conn, built_query)
            .await
            .with_context(|| "Generated by macro")

        //TODO Generate debug info in the macro from the input, with what are the parameters and their values
    }
    .context("")?;

    Ok(result)
}

#[always_context]
#[no_context]
async fn _test_insert() -> anyhow::Result<()> {
    let mut fake_conn = never_any::<Connection<TestDriver, DatabaseInternalDefault>>();

    let new_entry = ExampleTable {
        id: 1,
        field0: "test".to_string(),
        field1: "example".to_string(),
        field2: 123,
        field3: 456,
        field4: 7,
    };

    // query!(&mut fake_conn, INSERT INTO ExampleTable VALUES {new_entry});

    // query! macro output
    {
        //TODO Security checks
        // Imports
        use futures::FutureExt;

        async fn __easy_sql_perform<'a, T: Insert<'a, ExampleTable, TestDriver>>(
            exec: impl sqlx::Executor<'a, Database = crate::InternalDriver<TestDriver>>,
            to_insert: T,
        ) -> anyhow::Result<crate::DriverQueryResult<TestDriver>> {
            let mut args = DriverArguments::<TestDriver>::default();
            let mut query = "INSERT INTO ".to_string();
            let mut current_arg_n = 0;
            let mut _easy_sql_d = TestDriver::identifier_delimiter();

            // Build query
            query.push_str(ExampleTable::table_name());
            query.push_str(" (");

            let columns = <ExampleTable as Insert<ExampleTable, TestDriver>>::insert_columns();
            for (i, col) in columns.iter().enumerate() {
                if i > 0 {
                    query.push_str(", ");
                }
                query.push_str(&format!("{_easy_sql_d}{col}{_easy_sql_d}"));
            }

            query.push_str(") VALUES");

            let (new_args, count) = to_insert
                .insert_values_sqlx(args)
                .context("Failed to get insert values")?;
            args = new_args;

            for _ in 0..count {
                query.push_str(" (");

                for i in 0..columns.len() {
                    query.push_str(&TestDriver::parameter_placeholder(current_arg_n + i));
                    query.push_str(",");
                }
                current_arg_n += columns.len();
                query.pop(); //Remove last comma

                query.push_str("),");
            }
            query.pop(); //Remove last comma

            //Build and execute query
            let mut builder = sqlx::QueryBuilder::with_arguments(query, args);

            let built_query = builder.build();

            built_query
                .execute(exec)
                .await
                .context("Failed to execute insert query")
            //TODO Generate debug info in the macro from the input, with what are the parameters and their values
        }

        __easy_sql_perform(&mut *fake_conn, new_entry)
            .map(|r| r.with_context(|| "Generated by macro"))
    }
    .await
    .context("")?;

    Ok(())
}

#[always_context]
#[no_context]
async fn _test_update() -> anyhow::Result<()> {
    let mut fake_conn = never_any::<Connection<TestDriver, DatabaseInternalDefault>>();

    let data_update = ExampleTable {
        id: 1,
        field0: "updated_test".to_string(),
        field1: "updated_example".to_string(),
        field2: 456,
        field3: 789,
        field4: 10,
    };

    // query!(&mut fake_conn, UPDATE ExampleTable SET {data_update} WHERE id = {data_update.id});

    // query! macro output
    {
        //TODO Security checks
        // Imports
        use futures::FutureExt;
        use sqlx::Arguments;

        let mut args = DriverArguments::<TestDriver>::default();
        let mut query = "UPDATE ".to_string();
        let mut current_arg_n = 0;
        let mut _easy_sql_d = TestDriver::identifier_delimiter();

        // Build query
        query.push_str(ExampleTable::table_name());
        query.push_str(" SET ");

        args = (&data_update).updates_sqlx(args, &mut query, &mut current_arg_n)?;

        query.push_str(&format!(
            " WHERE {_easy_sql_d}id{_easy_sql_d} = {}",
            TestDriver::parameter_placeholder(current_arg_n)
        ));
        args.add(&data_update.id)
            .map_err(anyhow::Error::from_boxed)?;

        async fn execute<'a>(
            exec: impl sqlx::Executor<'a, Database = crate::InternalDriver<TestDriver>>,
            query: String,
            args: DriverArguments<'a, TestDriver>,
        ) -> Result<crate::DriverQueryResult<TestDriver>, sqlx::Error> {
            let mut builder = sqlx::QueryBuilder::with_arguments(query, args);

            let built_query = builder.build();

            built_query.execute(exec).await
        }

        execute(&mut *fake_conn, query, args).map(|r| r.with_context(|| "Generated by macro"))
        //TODO Generate debug info in the macro from the input, with what are the parameters and their values
    }
    .await
    .context("")?;

    Ok(())
}

#[always_context]
#[no_context]
async fn _test_delete() -> anyhow::Result<()> {
    let mut fake_conn = never_any::<Connection<TestDriver, DatabaseInternalDefault>>();

    let delete_id = 1;

    // query!(&mut fake_conn, DELETE FROM ExampleTable WHERE id = {delete_id});

    // query! macro output
    {
        //TODO Security checks
        // Imports
        use futures::FutureExt;
        use sqlx::Arguments;
        let mut args = DriverArguments::<TestDriver>::default();
        let mut query = "DELETE FROM ".to_string();
        let mut current_arg_n = 0;
        let mut _easy_sql_d = TestDriver::identifier_delimiter();

        // Build query
        query.push_str(ExampleTable::table_name());
        query.push_str(&format!(
            " WHERE {_easy_sql_d}id{_easy_sql_d} = {}",
            TestDriver::parameter_placeholder(current_arg_n)
        ));
        args.add(&delete_id).map_err(anyhow::Error::from_boxed)?;

        async fn execute<'a>(
            exec: impl sqlx::Executor<'a, Database = crate::InternalDriver<TestDriver>>,
            query: String,
            args: DriverArguments<'a, TestDriver>,
        ) -> Result<crate::DriverQueryResult<TestDriver>, sqlx::Error> {
            let mut builder = sqlx::QueryBuilder::with_arguments(query, args);

            let built_query = builder.build();

            built_query.execute(exec).await
        }

        execute(&mut *fake_conn, query, args).map(|r| r.with_context(|| "Generated by macro"))
        //TODO Generate debug info in the macro from the input, with what are the parameters and their values
    }
    .await
    .context("")?;

    Ok(())
}
