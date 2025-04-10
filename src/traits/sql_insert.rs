use anyhow::Context;
use easy_macros::macros::always_context;

use crate::SqlValueMaybeRef;

#[always_context]
pub trait SqlInsert<Table> {
    fn insert_columns() -> Vec<String>;
    fn insert_values(&self) -> anyhow::Result<Vec<Vec<SqlValueMaybeRef<'_>>>>;
}

#[always_context]
impl<T: SqlInsert<Table>, Table> SqlInsert<Table> for Vec<T> {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<SqlValueMaybeRef<'_>>>> {
        let mut result = Vec::new();
        for item in self.iter() {
            let values = item.insert_values()?;
            result.extend(values);
        }
        Ok(result)
    }
}

#[always_context]
impl<T: SqlInsert<Table>, Table> SqlInsert<Table> for &T {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<SqlValueMaybeRef<'_>>>> {
        (**self).insert_values()
    }
}

#[always_context]
impl<T: SqlInsert<Table>, Table> SqlInsert<Table> for [T] {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> anyhow::Result<Vec<Vec<SqlValueMaybeRef<'_>>>> {
        let mut result = Vec::new();
        for item in self.iter() {
            let values = item.insert_values()?;
            result.extend(values);
        }
        Ok(result)
    }
}
