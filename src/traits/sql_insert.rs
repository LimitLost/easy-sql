use easy_macros::macros::always_context;

use crate::SqlValueMaybeRef;

#[always_context]
pub trait SqlInsert<Table> {
    fn insert_columns() -> Vec<String>;
    fn insert_values(&self) -> Vec<Vec<SqlValueMaybeRef<'_>>>;
}

#[always_context]
impl<T: SqlInsert<Table>, Table> SqlInsert<Table> for Vec<T> {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> Vec<Vec<SqlValueMaybeRef<'_>>> {
        self.iter().flat_map(|item| item.insert_values()).collect()
    }
}

#[always_context]
impl<T: SqlInsert<Table>, Table> SqlInsert<Table> for [T] {
    fn insert_columns() -> Vec<String> {
        T::insert_columns()
    }

    fn insert_values(&self) -> Vec<Vec<SqlValueMaybeRef<'_>>> {
        self.iter().flat_map(|item| item.insert_values()).collect()
    }
}
