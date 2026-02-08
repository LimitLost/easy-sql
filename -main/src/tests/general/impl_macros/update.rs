use easy_macros::always_context;

use crate::{Driver, Update, traits::InternalDriver};

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
impl<'a, D: Driver> Update<'a, ExampleTableStruct, D> for ExampleStruct
where
    for<'x> String: sqlx::Encode<'x, InternalDriver<D>>,
    String: sqlx::Type<InternalDriver<D>>,
    for<'x> i32: sqlx::Encode<'x, InternalDriver<D>>,
    i32: sqlx::Type<InternalDriver<D>>,
    for<'x> i64: sqlx::Encode<'x, InternalDriver<D>>,
    i64: sqlx::Type<InternalDriver<D>>,
{
    fn updates(
        self,
        mut args_list: crate::traits::DriverArguments<'a, D>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<crate::traits::DriverArguments<'a, D>> {
        use sqlx::Arguments;

        let _ = |mut args_list: crate::traits::DriverArguments<'a, TestDriver>| {
            let _self = crate::macro_support::never_any::<Self>();

            args_list
                .add(_self.field1)
                .map_err(anyhow::Error::from_boxed)?;
            args_list
                .add(_self.field2)
                .map_err(anyhow::Error::from_boxed)?;
            args_list
                .add(_self.field3)
                .map_err(anyhow::Error::from_boxed)?;

            anyhow::Result::<()>::Ok(())
        };

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
        mut args_list: crate::traits::DriverArguments<'a, TestDriver>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<crate::traits::DriverArguments<'a, TestDriver>> {
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
