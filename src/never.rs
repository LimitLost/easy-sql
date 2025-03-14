///Used for compiler checks
pub fn never_fn<T>(_func: fn() -> T) {}

/// Used for compiler checks
///
/// Panics if called outside of `never_fn()` input block
///
pub fn never_any<T>() -> T {
    panic!("This function should never be called outside of never_fn() input block");
}
