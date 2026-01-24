use easy_macros::always_context;
use sqlx::Executor;

use crate::{Driver, DriverArguments};

#[always_context]
pub trait ToConvert<D: Driver> {
    async fn get<'a>(
        exec: impl Executor<'_, Database = D::InternalDriver>,
        query: sqlx::query::Query<'a, D::InternalDriver, DriverArguments<'a, D>>,
    ) -> anyhow::Result<Self>
    where
        Self: Sized;
}
#[always_context]
#[diagnostic::on_unimplemented(
    message = "Only types representing single row are allowed as output in query_lazy! calls."
)]
pub trait ToConvertSingle<D: Driver>: ToConvert<D> + sqlx::Row {}

#[always_context]
pub trait Output<Table, D: Driver>: Sized {
    type DataToConvert: ToConvert<D>;
    type UsedForChecks;

    fn select(current_query: &mut String);
    fn convert(data: Self::DataToConvert) -> anyhow::Result<Self>;
}

pub trait OutputData<Table> {
    type SelectProvider;
}

impl<T: OutputData<Table>, Table> OutputData<Table> for Vec<T> {
    type SelectProvider = T::SelectProvider;
}

impl<T: OutputData<Table>, Table> OutputData<Table> for Option<T> {
    type SelectProvider = T::SelectProvider;
}
impl<Table> OutputData<Table> for () {
    type SelectProvider = ();
}
