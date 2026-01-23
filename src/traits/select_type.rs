#[diagnostic::on_unimplemented(
    message = "Providing arguments for the selected output type is required. Tip: add parentheses with the inputs, after the selected output type, Example: {Self}(\"Example joined string start: \" || joined_column, 26)"
)]
pub trait NormalSelect {}

#[diagnostic::on_unimplemented(
    message = "Selected output type is not requesting any input arguments. Tip: remove parentheses with the inputs, after the selected output type"
)]
pub trait WithArgsSelect {}
