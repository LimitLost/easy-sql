//! Built-in SQL functions and operators supported by the easy-sql macros.
//!
//! The items below are recognized by [`query!`](crate::query) and [`query_lazy!`](crate::query_lazy)
//! across all drivers. Individual drivers opt into support by implementing the corresponding
//! marker traits (typically via [`impl_supports_fn`](crate::driver::impl_supports_fn),
//! [`impl_supports_fn_any`](crate::driver::impl_supports_fn_any), or manual impls).
//!
//! For custom SQL functions not listed here, use
//! [`custom_sql_function!`](crate::custom_sql_function).
//!
//! ## Built-in SQL functions
//! - Aggregates:
//!   - `COUNT` → [`SupportsCount`](crate::driver::functions::SupportsCount)
//!   - `SUM` → [`SupportsSum`](crate::driver::functions::SupportsSum)
//!   - `AVG` → [`SupportsAvg`](crate::driver::functions::SupportsAvg)
//!   - `MIN` → [`SupportsMin`](crate::driver::functions::SupportsMin)
//!   - `MAX` → [`SupportsMax`](crate::driver::functions::SupportsMax)
//! - Text:
//!   - `CONCAT` → [`SupportsConcat`](crate::driver::functions::SupportsConcat)
//!   - `UPPER` → [`SupportsUpper`](crate::driver::functions::SupportsUpper)
//!   - `LOWER` → [`SupportsLower`](crate::driver::functions::SupportsLower)
//!   - `LENGTH` → [`SupportsLength`](crate::driver::functions::SupportsLength)
//!   - `TRIM` → [`SupportsTrim`](crate::driver::functions::SupportsTrim)
//!   - `SUBSTRING` → [`SupportsSubstring`](crate::driver::functions::SupportsSubstring)
//!   - `SUBSTR` → [`SupportsSubstr`](crate::driver::functions::SupportsSubstr)
//! - Null handling:
//!   - `COALESCE` → [`SupportsCoalesce`](crate::driver::functions::SupportsCoalesce)
//!   - `NULLIF` → [`SupportsNullif`](crate::driver::functions::SupportsNullif)
//!   - `IFNULL` → [`SupportsIfnull`](crate::driver::functions::SupportsIfnull)
//! - Date/time:
//!   - `NOW` → [`SupportsNow`](crate::driver::functions::SupportsNow)
//!   - `DATE` → [`SupportsDate`](crate::driver::functions::SupportsDate)
//!   - `TIME` → [`SupportsTime`](crate::driver::functions::SupportsTime)
//!   - `DATETIME` → [`SupportsDatetime`](crate::driver::functions::SupportsDatetime)
//!   - `CURRENT_TIMESTAMP` → [`SupportsCurrentTimestamp`](crate::driver::functions::SupportsCurrentTimestamp)
//!   - `CURRENT_DATE` → [`SupportsCurrentDate`](crate::driver::functions::SupportsCurrentDate)
//!   - `CURRENT_TIME` → [`SupportsCurrentTime`](crate::driver::functions::SupportsCurrentTime)
//! - Math:
//!   - `ABS` → [`SupportsAbs`](crate::driver::functions::SupportsAbs)
//!   - `ROUND` → [`SupportsRound`](crate::driver::functions::SupportsRound)
//!   - `CEIL` → [`SupportsCeil`](crate::driver::functions::SupportsCeil)
//!   - `CEILING` → [`SupportsCeiling`](crate::driver::functions::SupportsCeiling)
//!   - `FLOOR` → [`SupportsFloor`](crate::driver::functions::SupportsFloor)
//!   - `POWER` → [`SupportsPower`](crate::driver::functions::SupportsPower)
//!   - `POW` → [`SupportsPow`](crate::driver::functions::SupportsPow)
//!   - `SQRT` → [`SupportsSqrt`](crate::driver::functions::SupportsSqrt)
//!   - `MOD` → [`SupportsMod`](crate::driver::functions::SupportsMod)
//! - Misc:
//!   - `CAST` → [`SupportsCast`](crate::driver::functions::SupportsCast)
//!   - `DISTINCT` → [`SupportsDistinct`](crate::driver::functions::SupportsDistinct)
//!
//! ## Built-in operators
//! - Boolean logic:
//!   - `AND` → [`SupportsAnd`](crate::driver::operators::SupportsAnd)
//!   - `OR` → [`SupportsOr`](crate::driver::operators::SupportsOr)
//! - Arithmetic:
//!   - `+` → [`SupportsAdd`](crate::driver::operators::SupportsAdd)
//!   - `-` → [`SupportsSub`](crate::driver::operators::SupportsSub)
//!   - `*` → [`SupportsMul`](crate::driver::operators::SupportsMul)
//!   - `/` → [`SupportsDiv`](crate::driver::operators::SupportsDiv)
//!   - `%` → [`SupportsModOperator`](crate::driver::operators::SupportsModOperator)
//! - String/json:
//!   - `||` → [`SupportsConcatOperator`](crate::driver::operators::SupportsConcatOperator)
//!   - `->` → [`SupportsJsonExtract`](crate::driver::operators::SupportsJsonExtract)
//!   - `->>` → [`SupportsJsonExtractText`](crate::driver::operators::SupportsJsonExtractText)
//! - Bitwise:
//!   - `&` → [`SupportsBitAnd`](crate::driver::operators::SupportsBitAnd)
//!   - `|` → [`SupportsBitOr`](crate::driver::operators::SupportsBitOr)
//!   - `<<` → [`SupportsBitShiftLeft`](crate::driver::operators::SupportsBitShiftLeft)
//!   - `>>` → [`SupportsBitShiftRight`](crate::driver::operators::SupportsBitShiftRight)
//! - Comparison:
//!   - `=` → [`SupportsEqual`](crate::driver::operators::SupportsEqual)
//!   - `!=` → [`SupportsNotEqual`](crate::driver::operators::SupportsNotEqual)
//!   - `>` → [`SupportsGreaterThan`](crate::driver::operators::SupportsGreaterThan)
//!   - `>=` → [`SupportsGreaterThanOrEqual`](crate::driver::operators::SupportsGreaterThanOrEqual)
//!   - `<` → [`SupportsLessThan`](crate::driver::operators::SupportsLessThan)
//!   - `<=` → [`SupportsLessThanOrEqual`](crate::driver::operators::SupportsLessThanOrEqual)
//!   - `LIKE` → [`SupportsLike`](crate::driver::operators::SupportsLike)
//!   - `IS NULL` → [`SupportsIsNull`](crate::driver::operators::SupportsIsNull)
//!   - `IS NOT NULL` → [`SupportsIsNotNull`](crate::driver::operators::SupportsIsNotNull)
//!   - `IN` → [`SupportsIn`](crate::driver::operators::SupportsIn)
//!   - `BETWEEN` → [`SupportsBetween`](crate::driver::operators::SupportsBetween)
