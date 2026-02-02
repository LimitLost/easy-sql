use easy_macros::always_context;
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};

use super::{
    DeleteQuery, ExistsQuery, InsertQuery, ProvidedDrivers, ReturningData, SelectQuery,
    UpdateQuery, group_by_clause, having_clause, limit_clause, order_by_clause, set_clause,
    where_clause,
};

struct ReturningArgData {
    arg_defs: Vec<TokenStream>,
    arg_tokens: Vec<TokenStream>,
}

impl ReturningData {
    fn build_arg_data(
        &self,
        sql_crate: &TokenStream,
        driver: &ProvidedDrivers,
        table_type: &syn::Type,
        binds: &mut Vec<TokenStream>,
        checks: &mut Vec<TokenStream>,
        param_counter: &mut usize,
        before_param_n: &mut TokenStream,
        before_format: &mut Vec<TokenStream>,
    ) -> ReturningArgData {
        let mut arg_defs = Vec::new();
        let mut arg_tokens = Vec::new();

        let output_type_ts = self.output_type.to_token_stream();

        if let Some(output_args) = &self.output_args {
            checks.push(quote! {
                let _ = || {
                    fn __easy_sql_assert_with_args<T: #sql_crate::WithArgsSelect>() {}
                    __easy_sql_assert_with_args::<<#output_type_ts as #sql_crate::OutputData<#table_type>>::SelectProvider>();
                };
            });

            for (idx, arg) in output_args.iter().enumerate() {
                let mut arg_format_params = Vec::new();
                let arg_sql_template = arg.into_query_string(
                    binds,
                    checks,
                    sql_crate,
                    driver,
                    param_counter,
                    &mut arg_format_params,
                    before_param_n,
                    before_format,
                    false,
                    false,
                    Some(&output_type_ts),
                    Some(&table_type.to_token_stream()),
                );
                let arg_ident = format_ident!("__easy_sql_returning_arg_{}", idx);
                let arg_def = if arg_format_params.is_empty() {
                    quote! {
                        let #arg_ident = #arg_sql_template.to_string();
                    }
                } else {
                    quote! {
                        let #arg_ident = format!(#arg_sql_template, #(#arg_format_params),*);
                    }
                };
                arg_defs.push(arg_def);
                arg_tokens.push(quote! { #arg_ident.as_str() });
            }
        } else {
            checks.push(quote! {
                let _ = || {
                    fn __easy_sql_assert_normal<T: #sql_crate::NormalSelect>() {}
                    __easy_sql_assert_normal::<<#output_type_ts as #sql_crate::OutputData<#table_type>>::SelectProvider>();
                };
            });
        }

        ReturningArgData {
            arg_defs,
            arg_tokens,
        }
    }
}

#[always_context]
pub fn generate_select(
    select: SelectQuery,
    connection: Option<&TokenStream>,
    driver: ProvidedDrivers,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let output = select.output;
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

    let output_arg_data = output.build_arg_data(
        sql_crate,
        &driver,
        &table_type,
        &mut binds,
        &mut checks,
        &mut param_counter,
        &mut before_param_n,
        &mut before_format,
    );
    let output_arg_defs = output_arg_data.arg_defs;
    let output_arg_tokens = output_arg_data.arg_tokens;

    let output_type = output.output_type;
    let output_args = output.output_args;
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
            &driver,
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
            &driver,
            Some(&output_type_ts),
            &table_type_tokens,
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
            &driver,
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
            &driver,
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
            &driver,
            &mut param_counter,
            &before_param_n,
        )
    }

    let lazy_mode_driver = if connection.is_none() {
        driver.single_driver()
    } else {
        None
    };

    if let Some(driver) = lazy_mode_driver {
        checks.push(quote! {
            {
                fn to_convert_single_impl<
                    Y: #sql_crate::ToConvertSingle<#driver>,
                    T: #sql_crate::Output<#table_type, #driver, DataToConvert = Y>,
                >(
                    _el: T,
                ) {
                }
                to_convert_single_impl(#macro_support::never_any::<#output_type>());
            }
        })
    }

    let debug_format_str = if lazy_mode_driver.is_some() {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let final_to_execute = if let Some(connection) = connection {
        quote! {
            let built_query = builder.build();

            // Execute query
            #macro_support::query_execute::<#table_type, #output_type, _>(&mut (#connection), built_query)
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
                                            <#output_type as #sql_crate::Output<#table_type, #lazy_mode_driver>>::convert(r)
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
                builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#lazy_mode_driver>>,
            }

            impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                fn fetch<'_easy_sql_e, E>(
                    &'_easy_sql_e mut self,
                    mut conn: &'_easy_sql_e mut E,
                ) -> impl #macro_support::Stream<
                    Item = #macro_support::Result<#output_type>,
                > + '_easy_sql_e
                where
                    &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#lazy_mode_driver> + '_easy_sql_e,
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
                    E: #sql_crate::EasyExecutor<#lazy_mode_driver> + '_easy_sql_e,
                    '_easy_sql_q: '_easy_sql_e,
                {
                    #fetch_internals_mut
                }
            }

            #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
        }
    };

    let driver_arguments = driver.arguments(sql_crate);
    let identifier_delimiter = driver.identifier_delimiter(sql_crate);
    let query_add_selected = if output_args.is_some() {
        driver.query_add_selected_with_args(sql_crate, &table_type, &output_type, output_arg_tokens)
    } else {
        driver.query_add_selected(sql_crate, &output_type, &table_type)
    };
    let main_table_name = driver.table_name(sql_crate, &table_type);
    let table_joins = driver.table_joins(sql_crate, &table_type);
    let parameter_placeholder_base = driver.parameter_placeholder_base(sql_crate);

    let async_block = if lazy_mode_driver.is_some() {
        quote! {}
    } else {
        quote! {async}
    };

    Ok(quote! {
        {
            #async_block {
                use {#sql_crate::ToConvert,#macro_support::{Context,Arguments}};

                // Safety checks closure
                let _ = |___t___: #table_type| {
                    #(#checks)*
                };

                let mut _easy_sql_args = #driver_arguments;
                let _easy_sql_d = #identifier_delimiter;
                #(#before_format)*
                let mut query = String::from(#query_base_str);
                #parameter_placeholder_base

                #(#output_arg_defs)*

                // Add output columns
                #query_add_selected

                query.push_str(&format!(" FROM {}", #main_table_name));
                // Handle potential table joins
                #table_joins

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
    connection: Option<&TokenStream>,
    driver: ProvidedDrivers,
    sql_crate: &TokenStream,
    macro_input: &str,
) -> anyhow::Result<TokenStream> {
    let table_type = insert.table_type;
    let values = insert.values;

    let macro_support = quote! {#sql_crate::macro_support};

    let lazy_mode_driver = if connection.is_none() {
        driver.single_driver()
    } else {
        None
    };

    let debug_format_str = if lazy_mode_driver.is_some() {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let (
        returning_select,
        execute_ending,
        lazy_struct,
        returning_arg_defs,
        returning_arg_binds,
        returning_before_format,
        returning_checks,
    ) = if let Some(returning) = insert.returning {
        let returning_type: syn::Type = returning.output_type.clone();
        let returning_has_args = returning.output_args.is_some();
        let mut returning_arg_binds = Vec::new();
        let mut returning_before_format = Vec::new();
        let mut returning_checks = Vec::new();
        let mut returning_param_counter = 0usize;
        let mut returning_before_param_n = quote! {current_arg_n + };

        let returning_arg_data = returning.build_arg_data(
            sql_crate,
            &driver,
            &table_type,
            &mut returning_arg_binds,
            &mut returning_checks,
            &mut returning_param_counter,
            &mut returning_before_param_n,
            &mut returning_before_format,
        );
        let returning_arg_defs = returning_arg_data.arg_defs;
        let returning_arg_tokens = returning_arg_data.arg_tokens;

        if let Some(driver) = lazy_mode_driver {
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

            let returning_select = if returning_has_args {
                quote! {
                    query.push_str(" RETURNING ");
                    query.push_str(&<#returning_type as #sql_crate::OutputData<#table_type>>::SelectProvider::__easy_sql_select::<#driver>(
                        _easy_sql_d,
                        #(#returning_arg_tokens),*
                    ));
                }
            } else {
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #driver>>::select(&mut query);
                }
            };

            (
                returning_select,
                quote! {
                    #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
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
                        to_convert_single_impl(#macro_support::never_any::<#returning_type>());
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
                returning_arg_defs,
                returning_arg_binds,
                returning_before_format,
                quote! {#(#returning_checks)*},
            )
        } else {
            let query_add_selected = if returning_has_args {
                driver.query_add_selected_with_args(
                    sql_crate,
                    &table_type,
                    &returning_type,
                    returning_arg_tokens,
                )
            } else {
                driver.query_add_selected(sql_crate, &returning_type, &table_type)
            };
            (
                quote! {
                    query.push_str(" RETURNING ");
                    #query_add_selected
                },
                quote! {
                    let built_query = builder.build();
                    #macro_support::query_execute::<#table_type,#returning_type,_>(&mut (#connection),built_query).await.with_context(|| format!(#debug_format_str, #macro_input))
                },
                quote! {},
                returning_arg_defs,
                returning_arg_binds,
                returning_before_format,
                quote! {#(#returning_checks)*},
            )
        }
    } else {
        if lazy_mode_driver.is_some() {
            anyhow::bail!(
                "INSERT queries in query_lazy! macro must have a RETURNING clause, use normal query! macro otherwise"
            );
        }
        (
            quote! {},
            quote! {
                let built_query = builder.build();
                #macro_support::query_execute_no_output(&mut (#connection),built_query).await.with_context(|| format!(#debug_format_str, #macro_input))
            },
            quote! {},
            Vec::new(),
            Vec::new(),
            Vec::new(),
            quote! {},
        )
    };

    let driver_arguments = driver.arguments(sql_crate);
    let identifier_delimiter = driver.identifier_delimiter(sql_crate);
    let main_table_name = driver.table_name(sql_crate, &table_type);
    let parameter_placeholder_base = driver.parameter_placeholder_base(sql_crate);
    let parameter_placeholder_fn = driver.parameter_placeholder_fn(sql_crate, Span::call_site());

    let query_insert_data = driver.query_insert_data(sql_crate, &table_type, values);

    let async_block = if lazy_mode_driver.is_some() {
        quote! {}
    } else {
        quote! {async}
    };

    Ok(quote! {
        {
            #lazy_struct

            #async_block {
                use #macro_support::{Arguments,Context};
                use #sql_crate::ToConvert;

                    let mut _easy_sql_args = #driver_arguments;
                    let mut query = String::from("INSERT INTO ");
                    let mut current_arg_n = 0;
                    let _easy_sql_d = #identifier_delimiter;
                    #parameter_placeholder_base

                    #returning_checks

                    query.push_str(#main_table_name);
                    query.push_str(" (");

                    let (columns, new_args, count) = #query_insert_data.with_context(|| format!(#debug_format_str, #macro_input))?;
                    for (i, col) in columns.iter().enumerate() {
                        if i > 0 {
                            query.push_str(", ");
                        }
                        query.push_str(&format!("{_easy_sql_d}{col}{_easy_sql_d}"));
                    }

                    query.push_str(") VALUES");
                    _easy_sql_args = new_args;

                    for _ in 0..count {
                        query.push_str(" (");
                        for i in 0..columns.len() {
                            query.push_str(&#parameter_placeholder_fn(current_arg_n + i));
                            query.push(',');
                        }
                        current_arg_n += columns.len();
                        query.pop(); // Remove last comma
                        query.push_str("),");
                    }
                    query.pop(); // Remove last comma

                    #(#returning_before_format)*
                    #(#returning_arg_defs)*
                    #returning_select

                    #(#returning_arg_binds)*

                    let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);

                    #execute_ending


            }
        }
    })
}

#[always_context]
pub fn generate_update(
    update: UpdateQuery,
    connection: Option<&TokenStream>,
    driver: ProvidedDrivers,
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
        &driver,
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
                &driver,
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
                &driver,
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

    let lazy_mode_driver = if connection.is_none() {
        driver.single_driver()
    } else {
        None
    };

    let debug_format_str = if lazy_mode_driver.is_some() {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let (returning_select, execute, returning_arg_defs) = if let Some(returning) = update.returning
    {
        let returning_type: syn::Type = returning.output_type.clone();
        let returning_has_args = returning.output_args.is_some();
        let returning_arg_data = returning.build_arg_data(
            sql_crate,
            &driver,
            &table_type,
            &mut all_binds,
            &mut checks,
            &mut param_counter,
            &mut before_param_n,
            &mut before_format,
        );
        let returning_arg_defs = returning_arg_data.arg_defs;
        let returning_arg_tokens = returning_arg_data.arg_tokens;

        if let Some(connection) = connection {
            let query_add_selected = if returning_has_args {
                driver.query_add_selected_with_args(
                    sql_crate,
                    &table_type,
                    &returning_type,
                    returning_arg_tokens,
                )
            } else {
                driver.query_add_selected(sql_crate, &returning_type, &table_type)
            };
            (
                quote! {
                    query.push_str(" RETURNING ");
                    #query_add_selected
                },
                quote! {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(&query, _easy_sql_args);
                    let built_query = builder.build();
                    #macro_support::query_execute::<#table_type, #returning_type, _>(&mut #connection, built_query)
                        .await
                        .with_context(|| format!(#debug_format_str, #macro_input))
                },
                returning_arg_defs,
            )
        } else {
            let fetch_internals = |executor: TokenStream| {
                quote! {
                        use #sql_crate::EasyExecutor as _;
                    self.builder.build().fetch(conn.#executor()).map(|r| {
                                    match r {
                                        Ok(r) => {
                                            let converted =
                                                <#returning_type as #sql_crate::Output<#table_type, #lazy_mode_driver>>::convert(r)
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
                        Y: #sql_crate::ToConvertSingle<#lazy_mode_driver>,
                        T: #sql_crate::Output<#table_type, #lazy_mode_driver, DataToConvert = Y>,
                    >(
                        _el: T,
                    ) {
                    }
                    to_convert_single_impl(#macro_support::never_any::<#returning_type>());
                }
            });
            let returning_select = if returning_has_args {
                quote! {
                    query.push_str(" RETURNING ");
                    query.push_str(&<#returning_type as #sql_crate::OutputData<#table_type>>::SelectProvider::__easy_sql_select::<#lazy_mode_driver>(
                        _easy_sql_d,
                        #(#returning_arg_tokens),*
                    ));
                }
            } else {
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #lazy_mode_driver>>::select(&mut query);
                }
            };

            (
                returning_select,
                quote! {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);

                    struct LazyQueryResult<'_easy_sql_a> {
                        builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#lazy_mode_driver>>,
                    }

                    impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                        fn fetch<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<
                            Item = #macro_support::Result<#returning_type>,
                        > + '_easy_sql_e
                        where
                            &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#lazy_mode_driver> + '_easy_sql_e,
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
                            E: #sql_crate::EasyExecutor<#lazy_mode_driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_mut
                        }
                    }

                    #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
                },
                returning_arg_defs,
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
                let query = #macro_support::query_with(&query, _easy_sql_args);
                #macro_support::query_execute_no_output(&mut (#connection), query)
                    .await
                    .with_context(|| format!(#debug_format_str, #macro_input))
            },
            Vec::new(),
        )
    };

    let driver_arguments = driver.arguments(sql_crate);
    let parameter_placeholder_base = driver.parameter_placeholder_base(sql_crate);
    let identifier_delimiter = driver.identifier_delimiter(sql_crate);
    let main_table_name = driver.table_name(sql_crate, &table_type);
    checks.push(quote! {
        let _ = || {
            fn __easy_sql_assert_not_joined<T: #sql_crate::NotJoinedTable>() {}
            __easy_sql_assert_not_joined::<#table_type>();
        };
    });

    let async_block = if lazy_mode_driver.is_some() {
        quote! {}
    } else {
        quote! {async}
    };

    Ok(quote! {
        {
            // Safety checks closure
            let _ = |___t___: #table_type| {
                #(#checks)*
            };

            #async_block {
                use #macro_support::{Context,Arguments};
                use #sql_crate::ToConvert;

                let mut _easy_sql_args = #driver_arguments;
                let _easy_sql_d = #identifier_delimiter;
                #parameter_placeholder_base
                #(#before_format)*
                let mut query = format!("UPDATE {}",#main_table_name);

                query.push_str(&format!(#format_str, #(#format_params),*));


                // Execute SET clause code
                #set_code

                // Build WHERE clause string
                #where_code

                // Add ALL parameter bindings
                #(#all_binds)*

                #(#returning_arg_defs)*
                #returning_select

                #execute
            }
        }
    })
}

#[always_context]
pub fn generate_delete(
    delete: DeleteQuery,
    connection: Option<&TokenStream>,
    driver: ProvidedDrivers,
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
            &driver,
            &mut param_counter,
            &mut before_param_n,
            &mut before_format,
            None, // Returning handling happens after the value is removed in the Sql engines
            &table_type_tokens,
        )
    }

    let lazy_mode_driver = if connection.is_none() {
        driver.single_driver()
    } else {
        None
    };

    let debug_format_str = if lazy_mode_driver.is_some() {
        "sql query_lazy! macro input: {}"
    } else {
        "sql query! macro input: {}"
    };

    let (returning_select, execute, returning_arg_defs) = if let Some(returning) = delete.returning
    {
        let returning_type: syn::Type = returning.output_type.clone();
        let returning_has_args = returning.output_args.is_some();
        let returning_arg_data = returning.build_arg_data(
            sql_crate,
            &driver,
            &table_type,
            &mut binds,
            &mut checks,
            &mut param_counter,
            &mut before_param_n,
            &mut before_format,
        );
        let returning_arg_defs = returning_arg_data.arg_defs;
        let returning_arg_tokens = returning_arg_data.arg_tokens;

        if let Some(connection) = connection {
            let query_add_selected = if returning_has_args {
                driver.query_add_selected_with_args(
                    sql_crate,
                    &table_type,
                    &returning_type,
                    returning_arg_tokens,
                )
            } else {
                driver.query_add_selected(sql_crate, &returning_type, &table_type)
            };
            (
                quote! {
                    query.push_str(" RETURNING ");
                    #query_add_selected
                },
                quote! {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(&query, _easy_sql_args);
                    let built_query = builder.build();
                    #macro_support::query_execute(&mut #connection, built_query)
                        .await
                        .with_context(|| format!(#debug_format_str, #macro_input))
                },
                returning_arg_defs,
            )
        } else {
            let fetch_internals = |executor: TokenStream| {
                quote! {
                        use #sql_crate::EasyExecutor as _;
                    self.builder.build().fetch(conn.#executor()).map(|r| {
                                    match r {
                                        Ok(r) => {
                                            let converted =
                                                <#returning_type as #sql_crate::Output<#table_type, #lazy_mode_driver>>::convert(r)
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
                        Y: #sql_crate::ToConvertSingle<#lazy_mode_driver>,
                        T: #sql_crate::Output<#table_type, #lazy_mode_driver, DataToConvert = Y>,
                    >(
                        _el: T,
                    ) {
                    }
                    to_convert_single_impl(#macro_support::never_any::<#returning_type>());
                }
            });
            let returning_select = if returning_has_args {
                quote! {
                    query.push_str(" RETURNING ");
                    query.push_str(&<#returning_type as #sql_crate::OutputData<#table_type>>::SelectProvider::__easy_sql_select::<#lazy_mode_driver>(
                        _easy_sql_d,
                        #(#returning_arg_tokens),*
                    ));
                }
            } else {
                quote! {
                    query.push_str(" RETURNING ");
                    <#returning_type as #sql_crate::Output<#table_type, #lazy_mode_driver>>::select(&mut query);
                }
            };

            (
                returning_select,
                quote! {
                    let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);

                    struct LazyQueryResult<'_easy_sql_a> {
                        builder: #macro_support::QueryBuilder<'_easy_sql_a, #sql_crate::InternalDriver<#lazy_mode_driver>>,
                    }

                    impl<'_easy_sql_q> LazyQueryResult<'_easy_sql_q> {
                        fn fetch<'_easy_sql_e, E>(
                            &'_easy_sql_e mut self,
                            mut conn: &'_easy_sql_e mut E,
                        ) -> impl #macro_support::Stream<
                            Item = #macro_support::Result<#returning_type>,
                        > + '_easy_sql_e
                        where
                            &'_easy_sql_e mut E: #sql_crate::EasyExecutor<#lazy_mode_driver> + '_easy_sql_e,
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
                            E: #sql_crate::EasyExecutor<#lazy_mode_driver> + '_easy_sql_e,
                            '_easy_sql_q: '_easy_sql_e,
                        {
                            #fetch_internals_mut
                        }
                    }

                    #macro_support::Result::<LazyQueryResult>::Ok(LazyQueryResult { builder })
                },
                returning_arg_defs,
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
                let mut builder = #macro_support::QueryBuilder::with_arguments(&query, _easy_sql_args);
                let built_query = builder.build();
                #macro_support::query_execute_no_output(&mut #connection, built_query)
                    .await
                    .with_context(|| format!(#debug_format_str, #macro_input))
            },
            Vec::new(),
        )
    };

    let driver_arguments = driver.arguments(sql_crate);
    let identifier_delimiter = driver.identifier_delimiter(sql_crate);
    let table_name = driver.table_name(sql_crate, &table_type);
    checks.push(quote! {
        let _ = || {
            fn __easy_sql_assert_not_joined<T: #sql_crate::NotJoinedTable>() {}
            __easy_sql_assert_not_joined::<#table_type>();
        };
    });
    let parameter_placeholder_base = driver.parameter_placeholder_base(sql_crate);

    let async_block = if lazy_mode_driver.is_some() {
        quote! {}
    } else {
        quote! {async}
    };

    Ok(quote! {
        {
            // Safety checks closure
            let _ = |___t___: #table_type| {
                #(#checks)*
            };

            #async_block {
                use #macro_support::{Context,Arguments};
                use #sql_crate::ToConvert;

                let mut _easy_sql_args = #driver_arguments;
                let _easy_sql_d = #identifier_delimiter;
                #parameter_placeholder_base
                #(#before_format)*

                let mut query = format!("DELETE FROM {}", #table_name);

                query.push_str(&format!(#format_str, #(#format_params),*));

                // Add WHERE parameter values
                {
                    #(#binds)*
                }

                #(#returning_arg_defs)*
                #returning_select

                #execute
            }
        }
    })
}

#[always_context]
pub fn generate_exists(
    exists: ExistsQuery,
    connection: &TokenStream,
    driver: ProvidedDrivers,
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
            &driver,
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
            &driver,
            None, // No output type in EXISTS
            &table_type_tokens,
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
            &driver,
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
            &driver,
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
            &driver,
            &mut param_counter,
            &before_param_n,
        )
    }

    format_str.push(')');

    let driver_arguments = driver.arguments(sql_crate);
    let identifier_delimiter = driver.identifier_delimiter(sql_crate);
    let table_name = driver.table_name(sql_crate, &table_type);
    let table_joins = driver.table_joins(sql_crate, &table_type);
    let parameter_placeholder_base = driver.parameter_placeholder_base(sql_crate);

    Ok(quote! {
        {
            // Safety checks closure
            let _ = |___t___: #table_type| {
                #(#checks)*
            };

            async {
                use #macro_support::{Context,Arguments};

                let mut _easy_sql_args = #driver_arguments;
                let _easy_sql_d = #identifier_delimiter;
                #parameter_placeholder_base
                #(#before_format)*
                let mut query = format!("SELECT EXISTS(SELECT 1 FROM {}",
                    #table_name
                );

                // Handle potential table joins
                #table_joins

                query.push_str(&format!(#format_str,
                    #(#format_params),*
                ));

                // Add WHERE, HAVING, and LIMIT parameter values
                {
                    #(#binds)*
                }

                let mut builder = #macro_support::QueryBuilder::with_arguments(query, _easy_sql_args);
                let built_query = builder.build();

                #macro_support::query_exists_execute(&mut (#connection), built_query)
                    .await
                    .with_context(|| format!("sql query! macro input: {}", #macro_input))
            }
        }
    })
}
