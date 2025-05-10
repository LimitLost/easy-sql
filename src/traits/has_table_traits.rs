use easy_macros::macros::always_context;

#[always_context]
/// Used to check if current struct representing tables has the table used in a query
pub trait HasTable<T> {}

#[always_context]
/// Used to check if columns referenced in output struct should be wrapped in option
pub trait HasTableJoined<T> {
    type MaybeOption<Y>;

    fn into_maybe_option<Y>(t: Y) -> Self::MaybeOption<Y>;
}
