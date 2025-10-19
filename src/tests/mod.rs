#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, DatabaseInternalDefault, Postgres as TestDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, DatabaseInternalDefault, Sqlite as TestDriver};

mod syntax;
mod syntax2;

mod how_to_sqlx;

mod impl_sql_insert;
mod impl_sql_output;
mod impl_sql_update;
mod impl_table;
// TODO Readme will be remade
// mod readme;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
#[cfg(not(all(feature = "postgres", feature = "sqlite")))]
mod delete;
mod insert;
mod select;
mod update;

mod sql_macro_modes;
