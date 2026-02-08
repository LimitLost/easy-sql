use easy_macros::always_context;

use crate::{Driver, traits::ToConvert};

#[always_context]
/// Indicates that a table type participates in the current query context.
///
/// Implemented by the table macros; avoid manual implementations.
#[diagnostic::on_unimplemented(
    message = "Type `{T}` is not a part of requested tables clause in this query, nor it is not an Output type (or using columns from Output type might not be supported in this position)"
)]
pub trait HasTable<T> {}

#[always_context]
/// Indicates that a joined table is optional and columns should be wrapped in `Option`.
///
/// Implemented by the join/output macros; avoid manual implementations.
pub trait HasTableJoined<T> {
    type MaybeOption<Y>;

    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>;
}

#[always_context]
/// Marker for row types supported by `query_lazy!` streaming output.
///
/// Implemented by driver integrations for their row types.
#[diagnostic::on_unimplemented(
    message = "Only types representing single row are allowed as output in query_lazy! calls."
)]
pub trait ToConvertSingle<D: Driver>: ToConvert<D> + sqlx::Row {}

#[diagnostic::on_unimplemented(
    message = "Providing arguments for the selected output type is required. Tip: add parentheses with the inputs, after the selected output type, Example: {Self}(\"Example joined string start: \" || joined_column, 26)"
)]
/// Marker for outputs without custom select arguments.
///
/// Implemented by the [`Output`](macro@crate::Output) derive macro.
pub trait NormalSelect {}

#[diagnostic::on_unimplemented(
    message = "Selected output type is not requesting any input arguments. Tip: remove parentheses with the inputs, after the selected output type"
)]
/// Marker for outputs that require custom select arguments.
///
/// Implemented by the [`Output`](macro@crate::Output) derive macro.
pub trait WithArgsSelect {}

/// Marker trait for tables that are not created via [`table_join!`](crate::table_join).
///
/// Implemented by the table macros; avoid manual implementations.
#[diagnostic::on_unimplemented(message = "UPDATE and DELETE queries do not support joined tables.")]
pub trait NotJoinedTable {}

/// Support trait providing fields information for query validation.
///
/// Implemented by the [`Output`](macro@crate::Output) derive macro and used internally by the query
/// macros.
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
