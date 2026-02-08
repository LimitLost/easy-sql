//! Marker traits for supported SQL operators.
//!
//! These traits gate operator usage inside [`query!`](crate::query) and
//! [`query_lazy!`](crate::query_lazy). Custom drivers implement the markers to describe which
//! operators their backend accepts.
//!
//! See [`supported`](crate::supported) module

use sql_macros::define_supports_operator_trait;

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
