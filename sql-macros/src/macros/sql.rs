use ::{
    quote::quote,
    syn::{self, parse::Parse},
};
use anyhow::Context;
use easy_macros::{helpers::parse_macro_input, macros::always_context};
use sql_compilation_data::CompilationData;

use crate::{
    macros_components::{
        column::SqlColumn, expr::SqlExpr, keyword, limit::SqlLimit, order_by::OrderBy, set::SetExpr,
    },
    sql_crate,
};

use super::WrappedInput;

/// Represents the three modes the sql! macro can operate in:
/// - Expression: Just a SQL expression (like sql_where!)
/// - SetClause: Update SET clause (like sql_set!)
/// - SelectClauses: Full SELECT clauses with WHERE, ORDER BY, etc. (original sql! behavior)
enum MacroMode {
    /// Just a SQL expression (e.g., `id = 5 AND name = "test"`)
    Expression(SqlExpr),
    /// SET clause for UPDATE statements (e.g., `field = value, name = "updated"`)
    SetClause(SetExpr),
    /// Full SELECT clauses (e.g., `WHERE id = 5 ORDER BY name LIMIT 10`)
    SelectClauses(SelectClauses),
}

struct SelectClauses {
    distinct: bool,
    where_: Option<SqlExpr>,
    order_by: Option<Vec<OrderBy>>,
    group_by: Option<Vec<SqlColumn>>,
    having: Option<SqlExpr>,
    limit: Option<SqlLimit>,
}

#[always_context]
impl SelectClauses {
    fn only_where(&self) -> bool {
        let Self {
            distinct,
            where_,
            order_by,
            group_by,
            having,
            limit,
        } = self;
        !distinct
            && where_.is_some()
            && order_by.is_none()
            && group_by.is_none()
            && having.is_none()
            && limit.is_none()
    }
}

#[always_context]
impl Parse for MacroMode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let is_set_clause = input.peek(keyword::set);

        if is_set_clause {
            input.parse::<keyword::set>()?;
            // Parse as SET clause
            return Ok(MacroMode::SetClause(input.parse()?));
        }

        // Check if this starts with a SELECT clause keyword (DISTINCT, WHERE, ORDER, GROUP, HAVING, LIMIT)
        let has_clause_keyword = input.peek(keyword::distinct)
            || input.peek(keyword::where_)
            || input.peek(keyword::order)
            || input.peek(keyword::group)
            || input.peek(keyword::having)
            || input.peek(keyword::limit);

        if has_clause_keyword {
            // Parse as SELECT clauses
            let mut distinct = false;
            let mut where_ = None;
            let mut order_by = None;
            let mut group_by = None;
            let mut having = None;
            let mut limit = None;

            while !input.is_empty() {
                let lookahead = input.lookahead1();
                if !distinct && lookahead.peek(keyword::distinct) {
                    input.parse::<keyword::distinct>()?;
                    distinct = true;
                    continue;
                }
                if where_.is_none() && lookahead.peek(keyword::where_) {
                    input.parse::<keyword::where_>()?;
                    where_ = Some(input.parse()?);
                    continue;
                }
                if order_by.is_none() && lookahead.peek(keyword::order) {
                    input.parse::<keyword::order>()?;
                    input.parse::<keyword::by>()?;
                    let mut order_by_list = Vec::new();
                    while !input.is_empty() {
                        let order_by_item: OrderBy = input.parse()?;
                        order_by_list.push(order_by_item);
                        if input.peek(syn::Token![,]) {
                            input.parse::<syn::Token![,]>()?;
                        } else {
                            break;
                        }
                    }
                    order_by = Some(order_by_list);
                    continue;
                }
                if group_by.is_none() && lookahead.peek(keyword::group) {
                    input.parse::<keyword::group>()?;
                    input.parse::<keyword::by>()?;
                    let mut group_by_list = Vec::new();
                    while !input.is_empty() {
                        let group_by_item: SqlColumn = input.parse()?;
                        group_by_list.push(group_by_item);
                        if input.peek(syn::Token![,]) {
                            input.parse::<syn::Token![,]>()?;
                        } else {
                            break;
                        }
                    }
                    group_by = Some(group_by_list);
                    continue;
                }
                if having.is_none() && lookahead.peek(keyword::having) {
                    input.parse::<keyword::having>()?;
                    having = Some(input.parse()?);
                    continue;
                }
                if limit.is_none() && lookahead.peek(keyword::limit) {
                    input.parse::<keyword::limit>()?;
                    limit = Some(input.parse()?);
                    continue;
                }
                return Err(lookahead.error());
            }
            return Ok(MacroMode::SelectClauses(SelectClauses {
                distinct,
                where_,
                order_by,
                group_by,
                having,
                limit,
            }));
        }

        // Otherwise, parse as a simple expression
        Ok(MacroMode::Expression(input.parse()?))
    }
}

#[always_context]
pub fn sql(item: proc_macro::TokenStream) -> anyhow::Result<proc_macro::TokenStream> {
    let input = parse_macro_input!(item as WrappedInput<MacroMode>);
    let input_table = input.table;
    let mode = input.input;

    let mut checks = Vec::new();
    let mut binds = Vec::new();
    let sql_crate = sql_crate();

    // Get the driver - either explicitly provided or from compilation data
    let driver = if let Some(explicit_driver) = input.driver {
        // Use explicitly provided driver
        quote! {#explicit_driver}
    } else {
        // Load compilation data to get default drivers
        let compilation_data = CompilationData::load(Vec::<String>::new(), false)?;

        // Get the driver from compilation data
        if compilation_data.default_drivers.is_empty() {
            anyhow::bail!(
                "No default drivers provided in the build script. Please configure drivers using sql_build::build() in build.rs, or specify a driver explicitly: sql!(<Driver> |Table| ...)"
            );
        } else if compilation_data.default_drivers.len() > 1 {
            anyhow::bail!(
                "Multiple drivers are configured in easy_sql.ron: {:?}. The sql! macro requires a single driver. Please configure only one driver in your build.rs, or specify the driver explicitly: sql!(<Driver> |Table| ...)",
                compilation_data.default_drivers
            );
        } else {
            let driver_str = &compilation_data.default_drivers[0];
            let driver_path: syn::Path = syn::parse_str(driver_str).with_context(|| {
                format!(
                    "easy_sql.ron is corrupted: Invalid driver name `{}`, expected valid Rust identifier",
                    driver_str
                )
            })?;
            quote! {#driver_path}
        }
    };

    let result = match mode {
        MacroMode::Expression(expr) => {
            // Expression mode: just returns a WhereClause (like sql_where!)

            let conditions_parsed =
                expr.into_tokens_with_checks(&mut checks, &mut binds, &sql_crate, true, &driver);

            if let Some(table_ty) = input_table {
                //Normal Mode

                quote! {
                    {
                        let _ = |___t___:#table_ty|{
                            #(#checks)*
                        };
                        |__easy_sql_builder: &mut #sql_crate::QueryBuilder<#driver>|{
                            // Safety: References to Variables created inside of the closure can't escape
                            // which is the only potentially unsafe situation here.
                            unsafe{
                                #(
                                    #binds
                                )*
                            }

                            Ok(#sql_crate::WhereClause{
                                conditions: #conditions_parsed
                            })
                        }
                    }
                }
            } else {
                //Debug info Mode
                quote! {
                    #sql_crate::WhereClause{
                        conditions: #conditions_parsed
                    }
                }
            }
        }
        MacroMode::SetClause(set_expr) => {
            // Set clause mode: returns an UpdateSetClause (like sql_set!)
            let mut set_updates = Vec::new();

            for (column, where_expr) in set_expr.updates {
                let where_expr_parsed = where_expr.into_tokens_with_checks(
                    &mut checks,
                    &mut binds,
                    &sql_crate,
                    false,
                    &driver,
                );
                let column_str = column.to_string();
                checks.push(quote! {
                    ___t___.#column;
                });
                set_updates.push(quote! {
                    (#column_str.to_string(), #where_expr_parsed)
                });
            }

            if let Some(table_ty) = input_table {
                // Normal Mode
                quote! {
                    {
                        let _ = |___t___:#table_ty|{
                            #(#checks)*
                        };
                        (
                            vec![#(#set_updates,)*],
                            |__easy_sql_builder: &mut #sql_crate::QueryBuilder<#driver>|{
                                // Safety: References to Variables created inside of the closure can't escape
                                // which is the only potentially unsafe situation here.
                                unsafe{
                                    #(
                                        #binds
                                    )*
                                }
                                Ok(())
                            }
                        )
                    }
                }
            } else {
                // Debug info Mode
                quote! {
                    vec![#(#set_updates,)*]
                }
            }
        }
        MacroMode::SelectClauses(input) => {
            // Select clauses mode: returns SelectClauses (original sql! behavior)
            if input.only_where() {
                let where_ = input.where_.unwrap().into_tokens_with_checks(
                    &mut checks,
                    &mut binds,
                    &sql_crate,
                    true,
                    &driver,
                );

                if let Some(table_ty) = input_table {
                    // Normal Mode
                    quote! {
                        {
                            let _ = |___t___:#table_ty|{
                                #(#checks)*
                            };
                            |__easy_sql_builder: &mut #sql_crate::QueryBuilder<#driver>|{
                                // Safety: References to Variables created inside of the closure can't escape
                                // which is the only potentially unsafe situation here.
                                unsafe{
                                    #(
                                        #binds
                                    )*
                                }
                                Ok(#sql_crate::WhereClause{
                                    conditions: #where_
                                })
                            }
                        }
                    }
                } else {
                    // Debug info Mode
                    quote! {
                        #sql_crate::WhereClause{
                            conditions: #where_
                        }
                    }
                }
            } else {
                let where_ = input
                    .where_
                    .map(|w| {
                        let tokens = w.into_tokens_with_checks(
                            &mut checks,
                            &mut binds,
                            &sql_crate,
                            true,
                            &driver,
                        );

                        quote! {Some(#sql_crate::WhereClause{
                            conditions:#tokens
                        })}
                    })
                    .unwrap_or_else(|| quote! {None});

                let group_by = input
                    .group_by
                    .map(|g| {
                        let tokens = g
                            .into_iter()
                            .map(|el| el.into_tokens_with_checks(&mut checks, &sql_crate));

                        quote! {Some(#sql_crate::GroupByClause{order_by: vec![#(#tokens),*]})}
                    })
                    .unwrap_or_else(|| quote! {None});

                let having = input
                    .having
                    .map(|h| {
                        let tokens = h.into_tokens_with_checks(
                            &mut checks,
                            &mut binds,
                            &sql_crate,
                            true,
                            &driver,
                        );

                        quote! {Some(#sql_crate::HavingClause{conditions: #tokens})}
                    })
                    .unwrap_or_else(|| quote! {None});

                let order_by = input
                    .order_by
                    .map(|o| {
                        let tokens = o
                            .into_iter()
                            .map(|el| el.into_tokens_with_checks(&mut checks, &sql_crate));

                        quote! {Some(#sql_crate::OrderByClause{order_by: vec![#(#tokens),*]})}
                    })
                    .unwrap_or_else(|| quote! {None});
                let limit = input
                    .limit
                    .map(|l| {
                        let tokens = l.into_tokens_with_checks(&mut checks, &sql_crate);

                        quote! {Some(#tokens)}
                    })
                    .unwrap_or_else(|| quote! {None});

                let distinct = input.distinct;

                if let Some(table_ty) = input_table {
                    // Normal Mode
                    quote! {
                        {
                            let _ = |___t___:#table_ty|{
                                #(#checks)*
                            };
                            |__easy_sql_builder: &mut #sql_crate::QueryBuilder<#driver>|{
                                // Safety: References to Variables created inside of the closure can't escape
                                // which is the only potentially unsafe situation here.
                                unsafe{
                                    #(
                                        #binds
                                    )*
                                }
                                Ok(#sql_crate::SelectClauses {
                                    distinct: #distinct,

                                    where_: #where_,
                                    group_by: #group_by,
                                    having: #having,
                                    order_by: #order_by,
                                    limit: #limit,
                                })
                            }
                        }
                    }
                } else {
                    // Debug info Mode
                    quote! {
                        #sql_crate::SelectClauses {
                            distinct: #distinct,

                            where_: #where_,
                            group_by: #group_by,
                            having: #having,
                            order_by: #order_by,
                            limit: #limit,
                        }
                    }
                }
            }
        }
    };

    // panic!("{}", result);

    Ok(result.into())
}
