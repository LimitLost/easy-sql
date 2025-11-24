use crate::{
    query_macro_components::{
        DeleteQuery, ExistsQuery, InsertQuery, QueryType, SelectQuery, UpdateQuery,
        group_by_clause, having_clause, limit_clause, order_by_clause, set_clause, where_clause,
    },
    sql_crate,
};

use anyhow::Context;
use easy_macros::always_context;
use proc_macro2::TokenStream;
use quote::quote;
use sql_compilation_data::CompilationData;
use syn::{self, parse::Parse};

/// Input structure for query! macro: connection, query_type
struct QueryInput {
    connection: syn::Expr,
    query: QueryType,
}

#[always_context]
impl Parse for QueryInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let connection = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let query = input.parse::<QueryType>()?;
        Ok(QueryInput { connection, query })
    }
}

/// Main entry point for query! macro
#[always_context]
pub fn query(input_raw: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let input_str = input_raw.to_string();
    let input = easy_macros::parse_macro_input!(input_raw as QueryInput);

    let connection = input.connection;

    // Load compilation data to get driver information
    let sql_crate = sql_crate();

    let compilation_data = CompilationData::load(Vec::<String>::new(), false).with_context(|| {
        "Failed to load compilation data for query! macro. Make sure easy_sql::build is called in build.rs"
    })?;

    let driver = if let Some(driver_str) = compilation_data.default_drivers.first() {
        let driver_path: syn::Path = syn::parse_str(driver_str)
            .with_context(|| format!("Failed to parse driver path: {}", driver_str))?;
        quote! {#driver_path}
    } else {
        return Err(anyhow::anyhow!(
            "No default driver found in compilation data. Please specify a driver in easy_sql.ron or build.rs"
        ));
    };

    let result = match input.query {
        QueryType::Select(select) => {
            generate_select(select.clone(), &connection, &driver, &sql_crate, &input_str)?
        }
        QueryType::Insert(insert) => {
            generate_insert(insert.clone(), &connection, &driver, &sql_crate, &input_str)?
        }
        QueryType::Update(update) => {
            generate_update(update.clone(), &connection, &driver, &sql_crate, &input_str)?
        }
        QueryType::Delete(delete) => {
            generate_delete(delete.clone(), &connection, &driver, &sql_crate, &input_str)?
        }
        QueryType::Exists(exists) => {
            generate_exists(exists.clone(), &connection, &driver, &sql_crate, &input_str)?
        }
    };

    Ok(result.into())
}

#[always_context]
fn generate_select(
    select: SelectQuery,
    connection: &syn::Expr,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let output_type = select.output_type;
    let table_type = select.table_type;
    let distinct = select.distinct;

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut binds = Vec::new();
    let mut param_counter = 0;

    let query_base_str = if distinct {
        "SELECT DISTINCT "
    } else {
        "SELECT "
    }
    .to_string();

    let mut format_str = " FROM {}".to_string();

    let mut format_params =
        vec![quote! { <#table_type as #sql_crate::Table<#driver>>::table_name() }];

    // Generate runtime code for WHERE clause
    if let Some(where_expr) = select.where_clause {
        where_clause(
            where_expr,
            &mut format_str,
            &mut format_params,
            &mut binds,
            &mut checks,
            sql_crate,
            driver,
            &mut param_counter,
            &quote! {},
        )
    }

    // Build GROUP BY clause code if present
    if let Some(group_by_list) = select.group_by {
        group_by_clause(
            group_by_list,
            &mut format_str,
            &mut format_params,
            sql_crate,
            &mut checks,
        )
    }

    // Generate runtime code for HAVING clause
    if let Some(having_expr) = select.having {
        having_clause(
            having_expr,
            &mut format_str,
            &mut format_params,
            &mut binds,
            &mut checks,
            sql_crate,
            driver,
            &mut param_counter,
            &quote! {},
        )
    }
    // Build ORDER BY clause code if present
    if let Some(order_by_list) = select.order_by {
        order_by_clause(
            order_by_list,
            &mut format_str,
            &mut format_params,
            sql_crate,
            &mut checks,
        )
    };

    // Build LIMIT clause code if present
    if let Some(limit) = select.limit {
        limit_clause(limit, &mut format_str, &mut format_params, &mut checks)
    }

    Ok(quote! {
        {
            async {
                use {#sql_crate::ToConvert,#macro_support::{Context,Arguments}};

                // Safety checks closure
                let _ = |___t___: #table_type| {
                    #(#checks)*
                };

                let mut _easy_sql_args = #sql_crate::DriverArguments::<#driver>::default();
                let _easy_sql_d = <#driver as #sql_crate::Driver>::identifier_delimiter();
                let mut query = String::from(#query_base_str);

                // Add output columns
                <#output_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);

                query.push_str(&format!(#format_str,
                    #(#format_params),*
                ));

                // Add WHERE and HAVING parameter values to args
                {
                    #(#binds)*
                }

                let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);
                let built_query = builder.build();

                // Execute query
                async fn execute<'a>(
                    exec: impl #macro_support::Executor<'a, Database = #sql_crate::InternalDriver<#driver>>,
                    query: #macro_support::Query<
                        'a,
                        #sql_crate::InternalDriver<#driver>,
                        #sql_crate::DriverArguments<'a, #driver>,
                    >,
                ) -> #macro_support::Result<#output_type> {
                    let raw_data = <#output_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                        exec, query
                    ).await.context("Output::DataToConvert::get failed")?;

                    let result = <#output_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")?;

                    Ok(result)
                }

                execute(&mut *#connection, built_query)
                    .await
                    .with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}

#[always_context]
fn generate_insert(
    insert: InsertQuery,
    connection: &syn::Expr,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = insert.table_type;
    let values = insert.values;

    let macro_support = quote! {#sql_crate::macro_support};

    let (returning_select_sqlx, result_type, execute_ending) = if let Some(returning_type) =
        insert.returning
    {
        (
            quote! {
                query.push_str(" RETURNING ");
                <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
            },
            quote! {#returning_type},
            quote! {
                let raw_data = <#returning_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                    exec, built_query
                ).await.context("DataToConvert::get failed")?;

                let result = <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")?;
            },
        )
    } else {
        (
            quote! {},
            quote! { #sql_crate::DriverQueryResult<#driver> },
            quote! {
                let result = built_query.execute(exec).await.context("QueryBuilder::build.execute failed")?;
            },
        )
    };

    Ok(quote! {
        {
            async {
                use #macro_support::{Context,FutureExt};
                use #sql_crate::ToConvert;

                async fn __easy_sql_perform<'a, T: #sql_crate::Insert<'a, #table_type, #driver>>(
                    exec: impl #macro_support::Executor<'a, Database = #sql_crate::InternalDriver<#driver>>,
                    to_insert: T,
                ) -> #macro_support::Result<#result_type> {
                    let mut _easy_sql_args = #sql_crate::DriverArguments::<#driver>::default();
                    let mut query = String::from("INSERT INTO ");
                    let mut current_arg_n = 0;
                    let _easy_sql_d = <#driver as #sql_crate::Driver>::identifier_delimiter();

                    query.push_str(<#table_type as #sql_crate::Table<#driver>>::table_name());
                    query.push_str(" (");

                    let columns = T::insert_columns();
                    for (i, col) in columns.iter().enumerate() {
                        if i > 0 {
                            query.push_str(", ");
                        }
                        query.push_str(&format!("{_easy_sql_d}{col}{_easy_sql_d}"));
                    }

                    query.push_str(") VALUES");

                    let (new_args, count) = to_insert
                        .insert_values_sqlx(_easy_sql_args)
                        .context("Insert::insert_values_sqlx failed")?;
                    _easy_sql_args = new_args;

                    for _ in 0..count {
                        query.push_str(" (");
                        for i in 0..columns.len() {
                            query.push_str(&<#driver as #sql_crate::Driver>::parameter_placeholder(current_arg_n + i));
                            query.push(',');
                        }
                        current_arg_n += columns.len();
                        query.pop(); // Remove last comma
                        query.push_str("),");
                    }
                    query.pop(); // Remove last comma

                    #returning_select_sqlx

                    let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);
                    let built_query = builder.build();

                    #execute_ending

                    Ok(result)
                }

                __easy_sql_perform(&mut *#connection, #values)
                    .await.with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}

#[always_context]
fn generate_update(
    update: UpdateQuery,
    connection: &syn::Expr,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type: syn::Type = update.table_type;
    let set_clause_data = update.set_clause;

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut all_binds = Vec::new();
    let mut param_counter = 0;

    let mut format_str = "UPDATE {}".to_string();
    let mut format_params =
        vec![quote! { <#table_type as #sql_crate::Table<#driver>>::table_name() }];

    // Process SET clause first
    let (set_code, before_param_n) = set_clause(
        set_clause_data,
        &mut format_str,
        &mut format_params,
        sql_crate,
        driver,
        &mut param_counter,
        &mut all_binds,
        &mut checks,
    );

    // Process WHERE clause with compile-time SQL generation
    let where_code = if let Some(where_expr) = update.where_clause {
        if let Some(before_param_n) = before_param_n {
            let mut clause_format_str = String::new();
            let mut clause_format_params = Vec::new();
            where_clause(
                where_expr,
                &mut clause_format_str,
                &mut clause_format_params,
                &mut all_binds,
                &mut checks,
                sql_crate,
                driver,
                &mut param_counter,
                &before_param_n,
            );

            quote! {
                query.push_str(&format!(#clause_format_str,
                    #(#clause_format_params),*
                ));
            }
        } else {
            where_clause(
                where_expr,
                &mut format_str,
                &mut format_params,
                &mut all_binds,
                &mut checks,
                sql_crate,
                driver,
                &mut param_counter,
                &quote! {},
            );
            quote! {}
        }
    } else {
        quote! {}
    };

    let (returning_select_sqlx, result_type, execute_insides) = if let Some(returning_type) =
        update.returning
    {
        (
            quote! {
                query.push_str(" RETURNING ");
                <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
            },
            quote! {#returning_type},
            quote! {
                let mut _easy_sql_builder = #macro_support::QueryBuilder::with_arguments(query_string, args);
                let _easy_sql_query = _easy_sql_builder.build();

                let raw_data = <#returning_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                    exec, _easy_sql_query
                ).await.context("Output::DataToConvert::get failed")?;

                <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")
            },
        )
    } else {
        (
            quote! {},
            quote! { #sql_crate::DriverQueryResult<#driver> },
            quote! {
                let mut builder = #macro_support::QueryBuilder::with_arguments(query_string, args);
                let built_query = builder.build();
                built_query.execute(exec).await.context("QueryBuilder::build.execute failed")
            },
        )
    };

    Ok(quote! {
        {
            // Safety checks closure
            let _ = |___t___: #table_type| {
                #(#checks)*
            };

            async {
                use #macro_support::{Context,FutureExt,Arguments};
                use #sql_crate::ToConvert;

                let mut _easy_sql_args = #sql_crate::DriverArguments::<#driver>::default();
                let _easy_sql_d = <#driver as #sql_crate::Driver>::identifier_delimiter();
                let mut query = format!(#format_str, #(#format_params),*);

                // Execute SET clause code
                #set_code

                // Build WHERE clause string
                #where_code

                // Add ALL parameter bindings
                #(#all_binds)*

                #returning_select_sqlx

                //Debug
                println!("Final query: {}", query);

                async fn execute<'a>(
                    exec: impl #macro_support::Executor<'a, Database = #sql_crate::InternalDriver<#driver>>,
                    query_string: String,
                    args: #sql_crate::DriverArguments<'a, #driver>,
                ) -> #macro_support::Result<#result_type> {
                    #execute_insides
                }

                execute(&mut *#connection, query, _easy_sql_args)
                    .await
                    .with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}

#[always_context]
fn generate_delete(
    delete: DeleteQuery,
    connection: &syn::Expr,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = delete.table_type;

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut binds = Vec::new();
    let mut param_counter = 0;

    let mut format_str = "DELETE FROM {}".to_string();
    let mut format_params =
        vec![quote! { <#table_type as #sql_crate::Table<#driver>>::table_name() }];

    // Generate runtime code for WHERE clause
    if let Some(where_expr) = delete.where_clause {
        where_clause(
            where_expr,
            &mut format_str,
            &mut format_params,
            &mut binds,
            &mut checks,
            sql_crate,
            driver,
            &mut param_counter,
            &quote! {},
        )
    }

    let (returning_select_sqlx, result_type, execute_ending) = if let Some(returning_type) =
        delete.returning
    {
        (
            quote! {
                query.push_str(" RETURNING ");
                <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
            },
            quote! {#returning_type},
            quote! {
                let raw_data = <#returning_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                    exec, query
                ).await.context("DataToConvert::get failed")?;

                let result = <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")?;
                Ok(result)
            },
        )
    } else {
        (
            quote! {},
            quote! { #sql_crate::DriverQueryResult<#driver> },
            quote! {
                query.execute(exec).await.context("QueryBuilder::build.execute failed")
            },
        )
    };

    Ok(quote! {
        {
            // Safety checks closure
            let _ = |___t___: #table_type| {
                #(#checks)*
            };

            async {
                use #macro_support::{Context,FutureExt,Arguments};
                use #sql_crate::ToConvert;

                let mut _easy_sql_args = #sql_crate::DriverArguments::<#driver>::default();
                let _easy_sql_d = <#driver as #sql_crate::Driver>::identifier_delimiter();
                let mut query = format!(#format_str,
                    #(#format_params),*
                );

                // Add WHERE parameter values
                {
                    #(#binds)*
                }

                #returning_select_sqlx

                let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);
                let built_query = builder.build();

                async fn execute<'a>(
                    exec: impl #macro_support::Executor<'a, Database = #sql_crate::InternalDriver<#driver>>,
                    query: #macro_support::Query<
                        'a,
                        #sql_crate::InternalDriver<#driver>,
                        #sql_crate::DriverArguments<'a, #driver>,
                    >,
                ) -> #macro_support::Result<#result_type> {
                    #execute_ending
                }

                execute(&mut *#connection, built_query)
                    .await
                    .with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}

#[always_context]
fn generate_exists(
    exists: ExistsQuery,
    connection: &syn::Expr,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = exists.table_type;

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut binds = Vec::new();
    let mut param_counter = 0;

    let mut format_str = "SELECT EXISTS(SELECT 1 FROM {}".to_string();
    let mut format_params =
        vec![quote! { <#table_type as #sql_crate::Table<#driver>>::table_name() }];

    // Generate runtime code for WHERE clause
    if let Some(where_expr) = exists.where_clause {
        where_clause(
            where_expr,
            &mut format_str,
            &mut format_params,
            &mut binds,
            &mut checks,
            sql_crate,
            driver,
            &mut param_counter,
            &quote! {},
        )
    };

    format_str.push(')');

    Ok(quote! {
        {
            // Safety checks closure
            let _ = |___t___: #table_type| {
                #(#checks)*
            };

            async {
                use #macro_support::{Context,Arguments};

                let mut _easy_sql_args = #sql_crate::DriverArguments::<#driver>::default();
                let _easy_sql_d = <#driver as #sql_crate::Driver>::identifier_delimiter();
                let mut query = format!(#format_str,
                    #(#format_params),*
                );

                // Add WHERE parameter values
                {
                    #(#binds)*
                }

                let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);
                let built_query = builder.build();

                async fn execute<'a>(
                    exec: impl #macro_support::Executor<'a, Database = #sql_crate::InternalDriver<#driver>>,
                    query: #macro_support::Query<
                        'a,
                        #sql_crate::InternalDriver<#driver>,
                        #sql_crate::DriverArguments<'a, #driver>,
                    >,
                ) -> #macro_support::Result<bool> {
                    let row = query.fetch_one(exec).await.context("sqlx::Query::fetch_one failed")?;
                    let exists: bool = <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&row, 0).context("SqlxRow::try_get failed")?;

                    Ok(exists)
                }

                execute(&mut *#connection, built_query)
                    .await
                    .with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}
