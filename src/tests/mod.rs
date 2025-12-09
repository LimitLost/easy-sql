#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, DatabaseInternalDefault, Postgres as TestDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, DatabaseInternalDefault, Sqlite as TestDriver};

mod _logger;
pub use _logger::*;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod impl_macros;
// TODO Readme will be remade
// mod readme;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod delete;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod insert;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod select;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod update;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod sql_macro_modes;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod query_macro;
