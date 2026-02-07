use easy_macros::always_context;
use sql_macros::{define_supports_fn_trait, define_supports_operator_trait};

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

define_supports_fn_trait!(SupportsCount, "COUNT");
define_supports_fn_trait!(SupportsSum, "SUM");
define_supports_fn_trait!(SupportsAvg, "AVG");
define_supports_fn_trait!(SupportsMin, "MIN");
define_supports_fn_trait!(SupportsMax, "MAX");

define_supports_fn_trait!(SupportsConcat, "CONCAT");
define_supports_fn_trait!(SupportsUpper, "UPPER");
define_supports_fn_trait!(SupportsLower, "LOWER");
define_supports_fn_trait!(SupportsLength, "LENGTH");
define_supports_fn_trait!(SupportsTrim, "TRIM");
define_supports_fn_trait!(SupportsSubstring, "SUBSTRING");
define_supports_fn_trait!(SupportsSubstr, "SUBSTR");

define_supports_fn_trait!(SupportsCoalesce, "COALESCE");
define_supports_fn_trait!(SupportsNullif, "NULLIF");
define_supports_fn_trait!(SupportsIfnull, "IFNULL");

define_supports_fn_trait!(SupportsNow, "NOW");
define_supports_fn_trait!(SupportsDate, "DATE");
define_supports_fn_trait!(SupportsTime, "TIME");
define_supports_fn_trait!(SupportsDatetime, "DATETIME");
define_supports_fn_trait!(SupportsCurrentTimestamp, "CURRENT_TIMESTAMP");
define_supports_fn_trait!(SupportsCurrentDate, "CURRENT_DATE");
define_supports_fn_trait!(SupportsCurrentTime, "CURRENT_TIME");

define_supports_fn_trait!(SupportsAbs, "ABS");
define_supports_fn_trait!(SupportsRound, "ROUND");
define_supports_fn_trait!(SupportsCeil, "CEIL");
define_supports_fn_trait!(SupportsCeiling, "CEILING");
define_supports_fn_trait!(SupportsFloor, "FLOOR");
define_supports_fn_trait!(SupportsPower, "POWER");
define_supports_fn_trait!(SupportsPow, "POW");
define_supports_fn_trait!(SupportsSqrt, "SQRT");
define_supports_fn_trait!(SupportsMod, "MOD");

define_supports_fn_trait!(SupportsCast, "CAST");
define_supports_fn_trait!(SupportsDistinct, "DISTINCT");

define_supports_operator_trait!(SupportsAnd, "AND");
define_supports_operator_trait!(SupportsOr, "OR");
define_supports_operator_trait!(SupportsAdd, "+");
define_supports_operator_trait!(SupportsSub, "-");
define_supports_operator_trait!(SupportsMul, "*");
define_supports_operator_trait!(SupportsDiv, "/");
define_supports_operator_trait!(SupportsModOperator, "%");
define_supports_operator_trait!(SupportsConcatOperator, "||");
define_supports_operator_trait!(SupportsJsonExtract, "->");
define_supports_operator_trait!(SupportsJsonExtractText, "->>");
define_supports_operator_trait!(SupportsBitAnd, "&");
define_supports_operator_trait!(SupportsBitOr, "|");
define_supports_operator_trait!(SupportsBitShiftLeft, "<<");
define_supports_operator_trait!(SupportsBitShiftRight, ">>");
define_supports_operator_trait!(SupportsEqual, "=");
define_supports_operator_trait!(SupportsNotEqual, "!=");
define_supports_operator_trait!(SupportsGreaterThan, ">");
define_supports_operator_trait!(SupportsGreaterThanOrEqual, ">=");
define_supports_operator_trait!(SupportsLessThan, "<");
define_supports_operator_trait!(SupportsLessThanOrEqual, "<=");
define_supports_operator_trait!(SupportsLike, "LIKE");
define_supports_operator_trait!(SupportsIsNull, "IS NULL");
define_supports_operator_trait!(SupportsIsNotNull, "IS NOT NULL");
define_supports_operator_trait!(SupportsIn, "IN");
define_supports_operator_trait!(SupportsBetween, "BETWEEN");
