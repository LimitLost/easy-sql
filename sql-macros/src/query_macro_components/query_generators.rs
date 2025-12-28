use easy_macros::always_context;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use super::{
    DeleteQuery, ExistsQuery, InsertQuery, SelectQuery, UpdateQuery, group_by_clause,
    having_clause, limit_clause, order_by_clause, set_clause, where_clause,
};

#[always_context]
pub fn generate_select(
    select: SelectQuery,
    connection: Option<&syn::Expr>,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let output_type = select.output_type;
    let table_type = select.table_type;
    let table_type_tokens = table_type.to_token_stream();
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

    let mut format_str = "".to_string();

    let mut format_params = vec![];

    let mut before_param_n = quote! {};
    let mut before_format = Vec::new();

    // Convert output_type to TokenStream for passing to validation functions
    let output_type_ts = output_type.to_token_stream();

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
            &mut before_param_n,
            &mut before_format,
            Some(&output_type_ts),
            &table_type_tokens,
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
            &mut before_param_n,
            &mut before_format,
            Some(&output_type_ts),
            &table_type_tokens,
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
            &mut binds,
            driver,
            &mut param_counter,
            &mut before_param_n,
            &mut before_format,
            Some(&output_type_ts),
            &table_type_tokens,
        )
    };

    // Build LIMIT clause code if present
    if let Some(limit) = select.limit {
        limit_clause(
            limit,
            &mut format_str,
            &mut format_params,
            &mut checks,
            &mut binds,
            sql_crate,
            driver,
            &mut param_counter,
            &before_param_n,
        )
    }

    let lazy_mode = connection.is_none();

    if lazy_mode {
        checks.push(quote! {
            {
                fn to_convert_single_impl<
                    Y: #sql_crate::ToConvertSingle<#driver>,
                    T: #sql_crate::Output<#table_type, #driver, DataToConvert = Y>,
                >(
                    _el: T,
                ) {
                }
                to_convert_single_impl(never_any::<#output_type>());
            }
        })
    }

    let debug_format_str = if lazy_mode {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let final_to_execute = if let Some(connection) = connection {
        quote! {
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

            execute(#sql_crate::EasyExecutor::executor(&mut #connection), built_query)
                .await
                .with_context(|| format!(#debug_format_str, #macro_input))
        }
    } else {
        let fetch_internals = |executor: TokenStream| {
            quote! {
                    use #sql_crate::EasyExecutor as _;
                self.builder.build().fetch(conn.#executor()).map(|r| {
                                match r {
                                    Ok(r) => {
                                        let converted =
                                            <#output_type as #sql_crate::Output<#table_type, #driver>>::convert(r)
                                                .context("Output::convert failed")?;

                                        Ok(converted)
                                    }
                                    Err(err) => Err(#macro_support::Error::from(err)),
                                }
                                .with_context(|| format!(#debug_format_str, #macro_input))
                            })
            }
        };

        let fetch_internals_normal = fetch_internals(quote! {into_executor});
        let fetch_internals_mut = fetch_internals(quote! {executor});

        quote! {
            struct LazyQueryResult<'_easy_sql_a> {
                builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#driver>>,
            }

            impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                fn fetch<'_easy_sql_e, E>(
                    &'_easy_sql_e mut self,
                    mut conn: &'_easy_sql_e mut E,
                ) -> impl #macro_support::Stream<
                    Item = #macro_support::Result<ExprTestData>,
                > + '_easy_sql_e
                where
                    &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                    '_easy_sql_q: '_easy_sql_e,
                {
                    #fetch_internals_normal
                }
                /// Useful when you're passing a generic `&mut impl EasyExecutor` as an argument
                fn fetch_mut<'_easy_sql_e, E>(
                    &'_easy_sql_e mut self,
                    mut conn: &'_easy_sql_e mut E,
                ) -> impl #macro_support::Stream<Item = #macro_support::Result<#output_type>> + '_easy_sql_e
                where
                    E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                    '_easy_sql_q: '_easy_sql_e,
                {
                    #fetch_internals_mut
                }
            }

            #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
        }
    };

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
                #(#before_format)*
                let mut query = String::from(#query_base_str);

                // Add output columns
                <#output_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);

                query.push_str(&format!(" FROM {}",<#table_type as #sql_crate::Table<#driver>>::table_name()));
                // Handle potential table joins
                <#table_type as #sql_crate::Table<#driver>>::table_joins(&mut query);

                query.push_str(&format!(#format_str,
                    #(#format_params),*
                ));

                // Add WHERE and HAVING parameter values to args
                {
                    #(#binds)*
                }

                let mut builder = #macro_support::QueryBuilder::with_arguments(&query, _easy_sql_args);
                #final_to_execute
            }
        }
    })
}

#[always_context]
pub fn generate_insert(
    insert: InsertQuery,
    connection: Option<&syn::Expr>,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = insert.table_type;
    let values = insert.values;

    let macro_support = quote! {#sql_crate::macro_support};

    let lazy_mode = connection.is_none();

    let debug_format_str = if lazy_mode {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let (exec_input_param, exec_input_value) = if let Some(conn) = connection {
        (
            quote! {
                exec: &mut impl #sql_crate::EasyExecutor<#driver>,
            },
            quote! {&mut (#conn),},
        )
    } else {
        (quote! {}, quote! {})
    };

    let (returning_select_sqlx, result_type, execute_ending, lazy_struct) = if let Some(
        returning_type,
    ) = insert.returning
    {
        if lazy_mode {
            let fetch_internals = |executor: TokenStream| {
                quote! {
                        use #sql_crate::EasyExecutor as _;
                    self.builder.build().fetch(conn.#executor()).map(|r| {
                                    match r {
                                        Ok(r) => {
                                            let converted =
                                                <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(r)
                                                    .context("Output::convert failed")?;

                                            Ok(converted)
                                        }
                                        Err(err) => Err(#macro_support::Error::from(err)),
                                    }
                                    .with_context(|| format!(#debug_format_str, #macro_input))
                                })
                }
            };

            let fetch_internals_normal = fetch_internals(quote! {into_executor});
            let fetch_internals_mut = fetch_internals(quote! {executor});

            (
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
                },
                quote! {LazyQueryResult<'a>},
                quote! {
                    let result = LazyQueryResult { builder };
                },
                quote! {
                    let _ = || {
                        fn to_convert_single_impl<
                            Y: #sql_crate::ToConvertSingle<#driver>,
                            T: #sql_crate::Output<#table_type, #driver, DataToConvert = Y>,
                        >(
                            _el: T,
                        ) {
                        }
                        to_convert_single_impl(never_any::<#returning_type>());
                    };
                    struct LazyQueryResult<'_easy_sql_a> {
                        builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#driver>>,
                    }

                    impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                        fn fetch<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<
                            Item = #macro_support::Result<#returning_type>,
                        > + '_easy_sql_e
                        where
                            &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_normal
                        }

                        /// Useful when you're passing a generic `&mut impl EasyExecutor` as an argument
                        fn fetch_mut<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<Item = #macro_support::Result<#returning_type>> + '_easy_sql_e
                        where
                            E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_mut
                        }
                    }
                },
            )
        } else {
            (
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
                },
                quote! {#returning_type},
                quote! {
                    let built_query = builder.build();
                    let raw_data = <#returning_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                        exec.executor(), built_query
                    ).await.context("DataToConvert::get failed")?;

                    let result = <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")?;
                },
                quote! {},
            )
        }
    } else {
        if lazy_mode {
            anyhow::bail!(
                "INSERT queries in query_lazy! macro must have a RETURNING clause, use normal query! macro otherwise"
            );
        }
        (
            quote! {},
            quote! { #sql_crate::DriverQueryResult<#driver> },
            quote! {
                let built_query = builder.build();
                let result = built_query.execute(exec.executor()).await.context("QueryBuilder::build.execute failed")?;
            },
            quote! {},
        )
    };

    Ok(quote! {
        {
            #lazy_struct

            async {
                use #macro_support::{Context,FutureExt};
                use #sql_crate::ToConvert;

                async fn __easy_sql_perform<'a, T: #sql_crate::Insert<'a, #table_type, #driver>>(
                    #exec_input_param
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

                    #execute_ending

                    Ok(result)
                }

                __easy_sql_perform(#exec_input_value #values)
                    .await.with_context(|| format!(#debug_format_str, #macro_input))
            }
        }
    })
}

#[always_context]
pub fn generate_update(
    update: UpdateQuery,
    connection: Option<&syn::Expr>,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type: syn::Type = update.table_type;
    let table_type_tokens = table_type.to_token_stream();
    let set_clause_data = update.set_clause;

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut all_binds = Vec::new();
    let mut param_counter = 0;

    let mut format_str = "".to_string();
    let mut format_params = vec![];

    let mut before_param_n = quote! {};
    let mut before_format = Vec::new();

    // Process SET clause first
    let set_code = set_clause(
        set_clause_data,
        &mut format_str,
        &mut format_params,
        sql_crate,
        driver,
        &mut param_counter,
        &mut all_binds,
        &mut checks,
        &mut before_param_n,
        &mut before_format,
        None, // No output type in UPDATE
        &table_type_tokens,
    );

    // Process WHERE clause with compile-time SQL generation
    let where_code = if let Some(where_expr) = update.where_clause {
        if !before_param_n.is_empty() {
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
                &mut before_param_n,
                &mut before_format,
                None, // Returning handling happens after the value is SET in the Sql engines
                &table_type_tokens,
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
                &mut before_param_n,
                &mut before_format,
                None, // Returning handling happens after the value is SET in the Sql engines
                &table_type_tokens,
            );
            quote! {}
        }
    } else {
        quote! {}
    };

    let lazy_mode = connection.is_none();

    let debug_format_str = if lazy_mode {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let (returning_select_sqlx, execute) = if let Some(returning_type) = update.returning {
        if let Some(connection) = connection {
            (
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
                },
                quote! {
                    async fn execute<'a>(
                        exec: &mut impl #sql_crate::EasyExecutor<#driver>,
                        query_string: String,
                        args: #sql_crate::DriverArguments<'a, #driver>,
                    ) -> #macro_support::Result<#returning_type> {
                        let mut _easy_sql_builder = #macro_support::QueryBuilder::with_arguments(query_string, args);
                        let _easy_sql_query = _easy_sql_builder.build();

                        let raw_data = <#returning_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                            exec.executor(), _easy_sql_query
                        ).await.context("Output::DataToConvert::get failed")?;

                        <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")
                    }

                    execute(&mut #connection, query, _easy_sql_args)
                        .await
                        .with_context(|| format!(#debug_format_str, #macro_input))
                },
            )
        } else {
            let fetch_internals = |executor: TokenStream| {
                quote! {
                        use #sql_crate::EasyExecutor as _;
                    self.builder.build().fetch(conn.#executor()).map(|r| {
                                    match r {
                                        Ok(r) => {
                                            let converted =
                                                <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(r)
                                                    .context("Output::convert failed")?;

                                            Ok(converted)
                                        }
                                        Err(err) => Err(#macro_support::Error::from(err)),
                                    }
                                    .with_context(|| format!(#debug_format_str, #macro_input))
                                })
                }
            };

            let fetch_internals_normal = fetch_internals(quote! {into_executor});
            let fetch_internals_mut = fetch_internals(quote! {executor});

            checks.push(quote! {
                {
                    fn to_convert_single_impl<
                        Y: #sql_crate::ToConvertSingle<#driver>,
                        T: #sql_crate::Output<#table_type, #driver, DataToConvert = Y>,
                    >(
                        _el: T,
                    ) {
                    }
                    to_convert_single_impl(never_any::<#returning_type>());
                }
            });
            (
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
                },
                quote! {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);

                    struct LazyQueryResult<'_easy_sql_a> {
                        builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#driver>>,
                    }

                    impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                        fn fetch<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<
                            Item = #macro_support::Result<#returning_type>,
                        > + '_easy_sql_e
                        where
                            &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_normal
                        }

                        /// Useful when you're passing a generic `&mut impl EasyExecutor` as an argument
                        fn fetch_mut<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<Item = #macro_support::Result<#returning_type>> + '_easy_sql_e
                        where
                            E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_mut
                        }
                    }

                    #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
                },
            )
        }
    } else {
        let connection = if let Some(c) = connection {
            c
        } else {
            anyhow::bail!(
                "UPDATE queries in query_lazy! macro must have a RETURNING clause, use normal query! macro otherwise"
            );
        };
        (
            quote! {},
            quote! {
                async fn execute<'a>(
                    exec: &mut impl #sql_crate::EasyExecutor<#driver>,
                    query_string: String,
                    args: #sql_crate::DriverArguments<'a, #driver>,
                ) -> #macro_support::Result<#sql_crate::DriverQueryResult<#driver>> {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(query_string, args);
                    let built_query = builder.build();
                    built_query.execute(exec.executor()).await.context("QueryBuilder::build.execute failed")
                }

                execute(&mut #connection, query, _easy_sql_args)
                    .await
                    .with_context(|| format!(#debug_format_str, #macro_input))
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
                #(#before_format)*
                let mut query = format!("UPDATE {}",<#table_type as #sql_crate::Table<#driver>>::table_name());

                // Handle potential table joins
                <#table_type as #sql_crate::Table<#driver>>::table_joins(&mut query);

                query.push_str(&format!(#format_str, #(#format_params),*));


                // Execute SET clause code
                #set_code

                // Build WHERE clause string
                #where_code

                // Add ALL parameter bindings
                #(#all_binds)*

                #returning_select_sqlx

                #execute
            }
        }
    })
}

#[always_context]
pub fn generate_delete(
    delete: DeleteQuery,
    connection: Option<&syn::Expr>,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = delete.table_type;
    let table_type_tokens = table_type.to_token_stream();

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut binds = Vec::new();
    let mut param_counter = 0;

    let mut format_str = "".to_string();
    let mut format_params = vec![];

    let mut before_param_n = quote! {};
    let mut before_format = Vec::new();

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
            &mut before_param_n,
            &mut before_format,
            None, // Returning handling happens after the value is removed in the Sql engines
            &table_type_tokens,
        )
    }

    let lazy_mode = connection.is_none();

    let debug_format_str = if lazy_mode {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let (returning_select_sqlx, execute) = if let Some(returning_type) = delete.returning {
        if let Some(connection) = connection {
            (
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
                },
                quote! {
                    async fn execute<'a>(
                        exec: &mut impl #sql_crate::EasyExecutor<#driver>,
                        query_string: String,
                        args: #sql_crate::DriverArguments<'a, #driver>,
                    ) -> #macro_support::Result<#returning_type> {
                        let mut _easy_sql_builder = #macro_support::QueryBuilder::with_arguments(query_string, args);
                        let _easy_sql_query = _easy_sql_builder.build();

                        let raw_data = <#returning_type as #sql_crate::Output<#table_type, #driver>>::DataToConvert::get(
                            exec.executor(), _easy_sql_query
                        ).await.context("Output::DataToConvert::get failed")?;

                        <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(raw_data).context("Output::convert failed")
                    }

                    execute(&mut #connection, query, _easy_sql_args)
                        .await
                        .with_context(|| format!(#debug_format_str, #macro_input))
                },
            )
        } else {
            let fetch_internals = |executor: TokenStream| {
                quote! {
                        use #sql_crate::EasyExecutor as _;
                    self.builder.build().fetch(conn.#executor()).map(|r| {
                                    match r {
                                        Ok(r) => {
                                            let converted =
                                                <#returning_type as #sql_crate::Output<#table_type, #driver>>::convert(r)
                                                    .context("Output::convert failed")?;

                                            Ok(converted)
                                        }
                                        Err(err) => Err(#macro_support::Error::from(err)),
                                    }
                                    .with_context(|| format!(#debug_format_str, #macro_input))
                                })
                }
            };

            let fetch_internals_normal = fetch_internals(quote! {into_executor});
            let fetch_internals_mut = fetch_internals(quote! {executor});

            checks.push(quote! {
                {
                    fn to_convert_single_impl<
                        Y: #sql_crate::ToConvertSingle<#driver>,
                        T: #sql_crate::Output<#table_type, #driver, DataToConvert = Y>,
                    >(
                        _el: T,
                    ) {
                    }
                    to_convert_single_impl(never_any::<#returning_type>());
                }
            });
            (
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select_sqlx(&mut query);
                },
                quote! {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);

                    struct LazyQueryResult<'_easy_sql_a> {
                        builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#driver>>,
                    }

                    impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                        fn fetch<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<
                            Item = #macro_support::Result<#returning_type>,
                        > + '_easy_sql_e
                        where
                            &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_normal
                        }

                        /// Useful when you're passing a generic `&mut impl EasyExecutor` as an argument
                        fn fetch_mut<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<Item = #macro_support::Result<#returning_type>> + '_easy_sql_e
                        where
                            E: #sql_crate::EasyExecutor<#driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_mut
                        }
                    }

                    #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
                },
            )
        }
    } else {
        let connection = if let Some(c) = connection {
            c
        } else {
            anyhow::bail!(
                "DELETE queries in query_lazy! macro must have a RETURNING clause, use normal query! macro otherwise"
            );
        };
        (
            quote! {},
            quote! {
                async fn execute<'a>(
                    exec: &mut impl #sql_crate::EasyExecutor<#driver>,
                    query_string: String,
                    args: #sql_crate::DriverArguments<'a, #driver>,
                ) -> #macro_support::Result<#sql_crate::DriverQueryResult<#driver>> {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(query_string, args);
                    let built_query = builder.build();
                    built_query.execute(exec.executor()).await.context("QueryBuilder::build.execute failed")
                }

                execute(&mut #connection, query, _easy_sql_args)
                    .await
                    .with_context(|| format!(#debug_format_str, #macro_input))
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
                #(#before_format)*

                let mut query = format!("DELETE FROM {}",<#table_type as #sql_crate::Table<#driver>>::table_name());

                // Handle potential table joins
                <#table_type as #sql_crate::Table<#driver>>::table_joins(&mut query);

                query.push_str(&format!(#format_str, #(#format_params),*));

                // Add WHERE parameter values
                {
                    #(#binds)*
                }

                #returning_select_sqlx

                #execute
            }
        }
    })
}

#[always_context]
pub fn generate_exists(
    exists: ExistsQuery,
    connection: &syn::Expr,
    driver: &TokenStream,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = exists.table_type;
    let table_type_tokens = table_type.to_token_stream();

    let macro_support = quote! {#sql_crate::macro_support};

    let mut checks = Vec::new();
    let mut binds = Vec::new();
    let mut param_counter = 0;

    let mut format_str = "".to_string();
    let mut format_params = vec![];

    let mut before_param_n = quote! {};
    let mut before_format = Vec::new();

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
            &mut before_param_n,
            &mut before_format,
            None, // No output type in EXISTS
            &table_type_tokens,
        )
    }

    // Build GROUP BY clause code if present
    if let Some(group_by_list) = exists.group_by {
        group_by_clause(
            group_by_list,
            &mut format_str,
            &mut format_params,
            sql_crate,
            &mut checks,
        )
    }

    // Generate runtime code for HAVING clause
    if let Some(having_expr) = exists.having {
        having_clause(
            having_expr,
            &mut format_str,
            &mut format_params,
            &mut binds,
            &mut checks,
            sql_crate,
            driver,
            &mut param_counter,
            &mut before_param_n,
            &mut before_format,
            None, // No output type in EXISTS
            &table_type_tokens,
        )
    }

    // Build ORDER BY clause code if present
    if let Some(order_by_list) = exists.order_by {
        order_by_clause(
            order_by_list,
            &mut format_str,
            &mut format_params,
            sql_crate,
            &mut checks,
            &mut binds,
            driver,
            &mut param_counter,
            &mut before_param_n,
            &mut before_format,
            None, // EXISTS queries don't have output types
            &table_type_tokens,
        )
    }

    // Build LIMIT clause code if present
    if let Some(limit) = exists.limit {
        limit_clause(
            limit,
            &mut format_str,
            &mut format_params,
            &mut checks,
            &mut binds,
            sql_crate,
            driver,
            &mut param_counter,
            &before_param_n,
        )
    }

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
                #(#before_format)*
                let mut query = format!("SELECT EXISTS(SELECT 1 FROM {}",
                    <#table_type as #sql_crate::Table<#driver>>::table_name()
                );

                // Handle potential table joins
                <#table_type as #sql_crate::Table<#driver>>::table_joins(&mut query);

                query.push_str(&format!(#format_str,
                    #(#format_params),*
                ));

                // Add WHERE, HAVING, and LIMIT parameter values
                {
                    #(#binds)*
                }

                let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);
                let built_query = builder.build();

                async fn execute<'a>(
                    exec: &mut impl #sql_crate::EasyExecutor<#driver>,
                    query: #macro_support::Query<
                        'a,
                        #sql_crate::InternalDriver<#driver>,
                        #sql_crate::DriverArguments<'a, #driver>,
                    >,
                ) -> #macro_support::Result<bool> {
                    let row = query.fetch_one(exec.executor()).await.context("sqlx::Query::fetch_one failed")?;
                    let exists: bool = <#sql_crate::DriverRow<#driver> as #sql_crate::SqlxRow>::try_get(&row, 0).context("SqlxRow::try_get failed")?;

                    Ok(exists)
                }

                execute(&mut #connection, built_query)
                    .await
                    .with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}
