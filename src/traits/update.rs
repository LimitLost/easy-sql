use easy_macros::always_context;

use super::{Driver, DriverArguments};

#[always_context]
/// Update payload mapping for a table.
///
/// Prefer implementing this trait via the [`Update`](macro@crate::Update) derive macro; manual
/// implementations may need updates across releases.
pub trait Update<'a, Table, D: Driver>: Sized {
    fn updates(
        self,
        args_list: DriverArguments<'a, D>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<DriverArguments<'a, D>>;
}
