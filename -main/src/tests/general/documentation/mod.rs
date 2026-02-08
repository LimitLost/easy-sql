mod custom_sql_function_macro;
#[cfg(not(feature = "migrations"))]
mod database_setup_macro;
mod impl_supports_fn_macro;
mod insert_macro;
mod output_macro;
mod query_lazy_macro;
mod query_macro;
mod table_join_macro;
#[cfg(feature = "migrations")]
mod table_macro;
mod update_macro;

#[cfg(all(not(feature = "migrations"), feature = "sqlite"))]
mod mini_demo;
