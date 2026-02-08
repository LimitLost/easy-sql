use proc_macro2::TokenStream;
use quote::quote;

use crate::macros_components::{Expr, Limit, OrderBy, column::Column};

use super::{CollectedData, SetClause};

fn sql_expr_clause(expr: Expr, clause_name: &'static str, data: &mut CollectedData) {
    let sql_template = expr.into_query_string(
        data, false, false, // for_custom_select
    );
    data.format_str
        .push_str(&format!(" {clause_name} {sql_template}"));
}

pub fn where_clause(where_expr: Expr, data: &mut CollectedData) {
    sql_expr_clause(where_expr, "WHERE", data)
}

pub fn having_clause(having_expr: Expr, data: &mut CollectedData) {
    sql_expr_clause(having_expr, "HAVING", data)
}

pub fn group_by_clause(group_by_list: Vec<Column>, data: &mut CollectedData) {
    let clause_args = group_by_list
        .into_iter()
        .map(|gb| gb.into_query_string(data, false))
        .collect::<Vec<_>>()
        .join(", ");

    data.format_str
        .push_str(&format!(" GROUP BY {}", clause_args));
}

pub fn order_by_clause(order_by_list: Vec<OrderBy>, data: &mut CollectedData) {
    let clause_args = order_by_list
        .into_iter()
        .map(|ob| ob.into_query_string(data))
        .collect::<Vec<_>>()
        .join(", ");

    data.format_str
        .push_str(&format!(" ORDER BY {}", clause_args));
}

pub fn limit_clause(limit: Limit, data: &mut CollectedData) {
    let clause_args = limit.into_query_string(data);

    data.format_str.push_str(&format!(" LIMIT {}", clause_args));
}

pub fn set_clause(clause: SetClause, data: &mut CollectedData) -> TokenStream {
    match clause {
        SetClause::FromType(type_expr) => {
            data.format_str.push_str(" SET ");
            let query_update_data = data.driver.query_update_data(
                data.sql_crate,
                data.main_table_type
                    .expect("Update Table missing in SET Clause"),
                *type_expr,
            );
            let current_param_n = &data.current_param_n;
            let before_param_n = &data.before_param_n;
            let result = quote! {
                // Use Update trait's updates method to add SET arguments
                let mut current_arg_n = #current_param_n;
                _easy_sql_args = #query_update_data.context("Update::updates failed")?;
            };
            *data.before_param_n = quote! { current_arg_n + #before_param_n};
            *data.current_param_n = 0;
            result
        }
        SetClause::Expr(set_expr) => {
            // Generate SET clause with compile-time SQL generation
            let mut set_sql_parts = Vec::new();

            for (ident, expr) in set_expr.updates {
                let ident_str = ident.to_string();
                let value_sql = expr.into_query_string(
                    data, false, false, // for_custom_select
                );
                set_sql_parts.push(format!(
                    "{{_easy_sql_d}}{}{{_easy_sql_d}} = {}",
                    ident_str, value_sql
                ));
            }

            // Generate compile-time SQL template for SET clause
            let set_sql_template = set_sql_parts.join(", ");

            data.format_str
                .push_str(&format!(" SET {}", set_sql_template));

            quote! {}
        }
    }
}
