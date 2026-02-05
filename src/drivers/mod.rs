//! Built-in driver implementations.
//!
//! Each driver module exposes a driver marker type and a `Database` helper for connections.

#[cfg(feature = "postgres")]
/// **PostgreSQL** database driver integration.
pub mod postgres;
#[cfg(feature = "sqlite")]
/// **SQLite** database driver integration.
pub mod sqlite;

#[cfg(feature = "postgres")]
pub use postgres::Postgres;
#[cfg(feature = "sqlite")]
pub use sqlite::Sqlite;
