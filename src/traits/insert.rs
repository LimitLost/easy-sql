use anyhow::Context;
use easy_macros::always_context;

use crate::{Driver, DriverArguments, QueryBuilder};

#[always_context]
pub trait Insert<'a, Table, D: Driver> {
    fn insert_columns() -> Vec<String>;
    ///Returns number of inserted rows
    fn insert_values(self, builder: &mut QueryBuilder<'_, D>) -> anyhow::Result<usize>;

    fn insert_values_sqlx(
        self,
        args_list: DriverArguments<'a, D>,
    ) -> anyhow::Result<(DriverArguments<'a, D>, usize)>;
}

#[always_context]
impl<'a, Table, T: Insert<'a, Table, D>, D: Driver> Insert<'a, Table, D> for Vec<T> {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, D>) -> anyhow::Result<usize> {
        let mut item_count = 0;
        for item in self.into_iter() {
            item_count += item.insert_values(
                #[context(no)]
                builder,
            )?;
        }
        Ok(item_count)
    }

    fn insert_values_sqlx(
        self,
        args_list: DriverArguments<'a, D>,
    ) -> anyhow::Result<(DriverArguments<'a, D>, usize)> {
        let mut args = args_list;
        let mut item_count = 0;
        for item in self.into_iter() {
            let (new_args, new_count) = item.insert_values_sqlx(
                #[context(no)]
                args,
            )?;
            args = new_args;
            item_count += new_count;
        }
        Ok((args, item_count))
    }
}

#[always_context]
impl<'a, Table, T: Insert<'a, Table, D>, D: Driver> Insert<'a, Table, D> for &'a Vec<T>
where
    &'a T: Insert<'a, Table, D>,
{
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, D>) -> anyhow::Result<usize> {
        let mut item_count = 0;
        for item in self.iter() {
            item_count += item.insert_values(
                #[context(no)]
                builder,
            )?;
        }
        Ok(item_count)
    }

    fn insert_values_sqlx(
        self,
        args_list: DriverArguments<'a, D>,
    ) -> anyhow::Result<(DriverArguments<'a, D>, usize)> {
        let mut args = args_list;
        let mut item_count = 0;
        for item in self.into_iter() {
            let (new_args, new_count) = item.insert_values_sqlx(
                #[context(no)]
                args,
            )?;
            args = new_args;
            item_count += new_count;
        }
        Ok((args, item_count))
    }
}

#[always_context]
impl<'a, Table, T: Insert<'a, Table, D>, D: Driver> Insert<'a, Table, D> for &'a [T]
where
    &'a T: Insert<'a, Table, D>,
{
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(self, builder: &mut QueryBuilder<'_, D>) -> anyhow::Result<usize> {
        let mut item_count = 0;
        for item in self.iter() {
            item_count += item.insert_values(
                #[context(no)]
                builder,
            )?;
        }
        Ok(item_count)
    }

    fn insert_values_sqlx(
        self,
        args_list: DriverArguments<'a, D>,
    ) -> anyhow::Result<(DriverArguments<'a, D>, usize)> {
        let mut args = args_list;
        let mut item_count = 0;
        for item in self.iter() {
            let (new_args, new_count) = item.insert_values_sqlx(
                #[context(no)]
                args,
            )?;
            args = new_args;
            item_count += new_count;
        }
        Ok((args, item_count))
    }
}
