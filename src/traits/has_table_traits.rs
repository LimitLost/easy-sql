use easy_macros::always_context;

#[always_context]
/// Used to check if current struct representing tables has the table used in a query
#[diagnostic::on_unimplemented(
    message = "Type `{T}` is not a part of requested tables clause in this query, nor it is not an Output type (or using columns from Output type might not be supported in this position)"
)]
pub trait HasTable<T> {}

#[always_context]
/// Used to check if columns referenced in output struct should be wrapped in option
pub trait HasTableJoined<T> {
    type MaybeOption<Y>;

    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>;
}
