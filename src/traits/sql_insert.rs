use anyhow::Context;
use easy_macros::macros::always_context;

use crate::Driver;

#[always_context]
pub trait SqlInsert<Table, D: Driver> {
    fn insert_columns() -> Vec<String>;
    fn insert_values(&self) -> anyhow::Result<Vec<Vec<D::Value<'_>>>>;
}

#[always_context]
impl<Table, T: SqlInsert<Table, D>, D: Driver> SqlInsert<Table, D> for Vec<T> {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<D::Value<'_>>>> {
        let mut result = Vec::new();
        for item in self.iter() {
            let values = item.insert_values()?;
            result.extend(values);
        }
        Ok(result)
    }
}

#[always_context]
impl<Table, T: SqlInsert<Table, D>, D: Driver> SqlInsert<Table, D> for &T {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<D::Value<'_>>>> {
        (**self).insert_values()
    }
}

#[always_context]
impl<Table, T: SqlInsert<Table, D>, D: Driver> SqlInsert<Table, D> for [T] {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<D::Value<'_>>>> {
        let mut result = Vec::new();
        for item in self.iter() {
            let values = item.insert_values()?;
            result.extend(values);
        }
        Ok(result)
    }
}
