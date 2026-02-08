//! Marker traits for built-in SQL functions.
//!
//! These are used by [`query!`](crate::query) and [`query_lazy!`](crate::query_lazy) to validate
//! SQL function calls at compile time. Custom drivers typically implement these via
//! [`impl_supports_fn`](crate::driver::impl_supports_fn) or
//! [`impl_supports_fn_any`](crate::driver::impl_supports_fn_any).
//!
//! See [`supported`](crate::supported) module

use easy_sql_macros::define_supports_fn_trait;

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
