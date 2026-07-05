use anyhow::{Context, bail};
use easy_macros::always_context;

use super::{Driver, DriverArguments, Update};
use crate::macro_support::UPDATE_NO_ASSIGNMENTS_MARKER;

#[always_context(skip(!))]
fn merge_collection_updates<'a, Table, D, I, U>(
    items: I,
    mut args: DriverArguments<'a, D>,
    current_query: &mut String,
    parameter_n: &mut usize,
    collection_kind: &'static str,
) -> anyhow::Result<DriverArguments<'a, D>>
where
    D: Driver,
    I: IntoIterator<Item = U>,
    U: Update<'a, Table, D>,
    <D::InternalDriver as sqlx::Database>::Arguments<'a>: std::fmt::Debug,
{
    let mut saw_any_item = false;
    let mut appended_any_assignments = false;
    let mut item_assignments_sql = String::new();

    for (index, item) in items.into_iter().enumerate() {
        saw_any_item = true;
        item_assignments_sql.clear();

        let updated_args = item
            .updates(
                #[context(no)]
                args,
                &mut item_assignments_sql,
                parameter_n,
            )
            .with_context(|| {
                format!(
                    "Failed to build UPDATE SET assignments from {} item at index {}",
                    collection_kind, index
                )
            })?;
        args = updated_args;

        if item_assignments_sql.as_str() == UPDATE_NO_ASSIGNMENTS_MARKER {
            item_assignments_sql.clear();
            continue;
        }

        if !item_assignments_sql.is_empty() {
            if appended_any_assignments {
                current_query.push_str(", ");
            }
            current_query.reserve(item_assignments_sql.len());
            current_query.push_str(&item_assignments_sql);
            appended_any_assignments = true;
        }
    }

    if !saw_any_item {
        bail!(
            "UPDATE ... SET {{collection}} cannot be empty. Provide at least one update item (supported: Vec<T>, &Vec<T>, &[T])."
        );
    }

    if !appended_any_assignments {
        bail!(
            "UPDATE ... SET {{collection}} produced no assignments. Ensure at least one item generates SET assignments (for example, maybe-update fields must contain Some(...))."
        );
    }

    Ok(args)
}

#[always_context(skip(!))]
impl<'a, Table, T: Update<'a, Table, D>, D: Driver> Update<'a, Table, D> for Vec<T>
where
    <D::InternalDriver as sqlx::Database>::Arguments<'a>: std::fmt::Debug,
{
    fn updates(
        self,
        args_list: DriverArguments<'a, D>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<DriverArguments<'a, D>> {
        merge_collection_updates::<Table, D, _, _>(
            self,
            args_list,
            current_query,
            parameter_n,
            "Vec<T>",
        )
    }
}

#[always_context(skip(!))]
impl<'a, Table, T: Update<'a, Table, D>, D: Driver> Update<'a, Table, D> for &'a Vec<T>
where
    &'a T: Update<'a, Table, D>,
    <D::InternalDriver as sqlx::Database>::Arguments<'a>: std::fmt::Debug,
{
    fn updates(
        self,
        args_list: DriverArguments<'a, D>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<DriverArguments<'a, D>> {
        merge_collection_updates::<Table, D, _, _>(
            self.iter(),
            args_list,
            current_query,
            parameter_n,
            "&Vec<T>",
        )
    }
}

#[always_context(skip(!))]
impl<'a, Table, T: Update<'a, Table, D>, D: Driver> Update<'a, Table, D> for &'a [T]
where
    &'a T: Update<'a, Table, D>,
    <D::InternalDriver as sqlx::Database>::Arguments<'a>: std::fmt::Debug,
{
    fn updates(
        self,
        args_list: DriverArguments<'a, D>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<DriverArguments<'a, D>> {
        merge_collection_updates::<Table, D, _, _>(
            self.iter(),
            args_list,
            current_query,
            parameter_n,
            "&[T]",
        )
    }
}
