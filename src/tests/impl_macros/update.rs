use anyhow::Context;
use easy_macros::macros::always_context;

use crate::{Driver, Expr, QueryBuilder, Update};

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

#[always_context]
#[no_context]
impl<'a> Update<'a, ExampleTableStruct, TestDriver> for ExampleStruct {
    fn updates(
        self,
        builder: &mut QueryBuilder<'_, TestDriver>,
    ) -> anyhow::Result<Vec<(String, Expr)>> {
        crate::never::never_fn(|| {
            //Check for validity
            let update_instance = crate::never::never_any::<Self>();
            let mut table_instance = crate::never::never_any::<ExampleTableStruct>();

            table_instance.field1 = update_instance.field1;
            table_instance.field2 = update_instance.field2;
            table_instance.field3 = update_instance.field3;
        });
        // Fully safe because we pass by value, not by reference
        unsafe {
            builder
                .bind(self.field1)
                .context("Binding `field1` failed")?;
            builder.bind(self.field2)?;
            builder.bind(self.field3)?;
        }
        Ok(vec![
            ("field1".to_string(), crate::Expr::Value),
            ("field2".to_string(), crate::Expr::Value),
            ("field3".to_string(), crate::Expr::Value),
        ])
    }

    fn updates_sqlx(
        self,
        mut args_list: crate::DriverArguments<'a, TestDriver>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<crate::DriverArguments<'a, TestDriver>> {
        use sqlx::Arguments;
        args_list
            .add(self.field1)
            .map_err(anyhow::Error::from_boxed)?;
        args_list
            .add(self.field2)
            .map_err(anyhow::Error::from_boxed)?;
        args_list
            .add(self.field3)
            .map_err(anyhow::Error::from_boxed)?;

        let delimeter = TestDriver::identifier_delimiter();

        current_query.push_str(&format!(
            "{delimeter}field1{delimeter} = {}, {delimeter}field2{delimeter} = {}, {delimeter}field3{delimeter} = {}",
            TestDriver::parameter_placeholder(*parameter_n),
            TestDriver::parameter_placeholder(*parameter_n + 1),
            TestDriver::parameter_placeholder(*parameter_n + 2),
        ));
        *parameter_n += 3; //Increase parameter count

        Ok(args_list)
    }
}

#[allow(dead_code)]
struct ExampleStruct2 {
    field1: String,
    field2: Option<i32>,
    field3: Option<i64>,
}

#[always_context]
#[no_context]
impl<'a> Update<'a, ExampleTableStruct, TestDriver> for ExampleStruct2 {
    fn updates(
        self,
        builder: &mut QueryBuilder<'_, TestDriver>,
    ) -> anyhow::Result<Vec<(String, Expr)>> {
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
        updates.push(("field1".to_string(), crate::Expr::Value));
        // Fully safe because we pass by value, not by reference
        unsafe {
            builder
                .bind(&self.field1)
                .context("Binding `field1` failed")?;
            if let Some(field2) = &self.field2 {
                updates.push(("field2".to_string(), crate::Expr::Value));
                builder.bind(field2)?;
            }
            if let Some(field3) = &self.field3 {
                updates.push(("field3".to_string(), crate::Expr::Value));
                builder.bind(field3)?;
            }
        }
        Ok(updates)
    }

    fn updates_sqlx(
        self,
        mut args_list: crate::DriverArguments<'a, TestDriver>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<crate::DriverArguments<'a, TestDriver>> {
        use sqlx::Arguments;
        let mut first = true;

        let delimeter = TestDriver::identifier_delimiter();

        //field1 is always set
        if !first {
            current_query.push_str(", ");
        }
        current_query.push_str(&format!(
            "{delimeter}field1{delimeter} = {}",
            TestDriver::parameter_placeholder(*parameter_n),
        ));
        args_list
            .add(self.field1)
            .map_err(anyhow::Error::from_boxed)?;
        *parameter_n += 1;
        first = false;

        if let Some(field2) = self.field2 {
            if !first {
                current_query.push_str(", ");
            }
            current_query.push_str(&format!(
                "{delimeter}field2{delimeter} = {}",
                TestDriver::parameter_placeholder(*parameter_n),
            ));
            args_list.add(field2).map_err(anyhow::Error::from_boxed)?;
            *parameter_n += 1;
            first = false;
        }

        if let Some(field3) = self.field3 {
            if !first {
                current_query.push_str(", ");
            }
            current_query.push_str(&format!(
                "{delimeter}field3{delimeter} = {}",
                TestDriver::parameter_placeholder(*parameter_n),
            ));
            args_list.add(field3).map_err(anyhow::Error::from_boxed)?;
            *parameter_n += 1;
        }

        Ok(args_list)
    }
}
