use easy_macros::always_context;

use crate::Driver;

#[always_context]
/// Marker for drivers that support auto-increment with composite primary keys.
///
/// Implement for custom drivers when the capability is available.
#[diagnostic::on_unimplemented(
    message = "Driver `{Self}` does not support auto-increment columns when the table uses a composite primary key. Remove #[sql(auto_increment)] or use a single-column primary key for this driver."
)]
pub trait SupportsAutoIncrementCompositePrimaryKey: Driver {}

#[always_context]
/// Marker for drivers that allow tables without a primary key.
///
/// Implement for custom drivers when the capability is available.
#[diagnostic::on_unimplemented(
    message = "Driver `{Self}` requires a primary key for tables. Add #[sql(primary_key)] to at least one field."
)]
pub trait AllowsNoPrimaryKey: Driver {}

#[always_context]
/// Marker for drivers that allow multiple auto-increment columns in a table.
///
/// Implement for custom drivers when the capability is available.
#[diagnostic::on_unimplemented(
    message = "Driver `{Self}` does not support multiple auto-increment columns in the same table. Remove #[sql(auto_increment)] from all but one column."
)]
pub trait SupportsMultipleAutoIncrementColumns: Driver {}
