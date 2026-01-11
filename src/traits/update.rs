use easy_macros::always_context;

use crate::{Driver, DriverArguments};

#[always_context]
pub trait Update<'a, Table, D: Driver>: Sized {
    fn updates_sqlx(
        self,
        args_list: DriverArguments<'a, D>,
        current_query: &mut String,
        parameter_n: &mut usize,
    ) -> anyhow::Result<DriverArguments<'a, D>>;
}
