//! Marker traits used by the macros to enforce query constraints.
//!
//! These are generally implemented by derive/macros or driver integrations, not by hand.

pub(crate) mod driver;
pub use driver::*;

mod query;
pub use query::*;
