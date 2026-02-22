use std::fmt::Display;

use super::CollectedData;
use ::{
    proc_macro2::{self},
    syn::spanned::Spanned,
};
use easy_macros::always_context;
use quote::{IdentFragment, ToTokens, format_ident, quote, quote_spanned};
pub use super::operator::{NotChain, Operator};
pub use super::value::{Value, ValueIn};

fn add_operator_support_check<T>(
    data: &mut CollectedData,
    operator_in_trait_name: T,
) where T:IdentFragment,for<'a> &'a T: Display+IdentFragment{
    let sql_crate = data.sql_crate;
    let trait_ident= format_ident!("Supports{}", operator_in_trait_name);
    for driver_ty in data.driver.iter_for_checks() {
        data.checks.push(quote_spanned! {proc_macro2::Span::call_site()=>
            {
                fn __easy_sql_assert_supports_operator<T: #sql_crate::markers::operators::#trait_ident>() {}
                __easy_sql_assert_supports_operator::<#driver_ty>();
            }
        });
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Value(Box<Value>),
    Parenthesized(Box<Expr>),
    OperatorChain(NotChain, Box<Expr>, Vec<(NotChain, Operator, Expr)>),
    IsNull(Box<Value>),
    IsNotNull(Box<Value>),
    In(Box<Value>, Box<ValueIn>),
    Between(Box<Value>, Box<Value>, Box<Value>),
}

#[always_context]
impl Expr {
    ///`main_table_type` - None means we're inside of table join
    pub fn into_query_string(
        self,
        data: &mut CollectedData,
        inside_count_fn: bool,
        for_custom_select: bool,
    ) -> String {
        match self {
            Expr::Value(val) => val.into_query_string(
                data,
                inside_count_fn,
                for_custom_select,
            ),
            Expr::IsNull(val) => {
                add_operator_support_check(
                    data,
                    "IsNull",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                format!("{} IS NULL", val_sql)
            }
            Expr::IsNotNull(val) => {
                add_operator_support_check(
                    data,
                    "IsNotNull",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                format!("{} IS NOT NULL", val_sql)
            }
            Expr::Parenthesized(inner) => {
                let inner_sql = inner.into_query_string(
                    data,
                    inside_count_fn,
                    for_custom_select,
                );
                format!("({})", inner_sql)
            }
            Expr::OperatorChain(not_chain, first, rest) => {
                let mut result = format!(
                    "{}{}",
                    not_chain.into_query_string(),
                    first.into_query_string(
                        data,
                        false,
                        for_custom_select,
                    )
                );
                for (not_chain, op, expr) in rest.into_iter() {
                    let op_trait_name = match &op {
                        Operator::Mod|Operator::Concat => format!("{:?}Operator",op),
                        op=>format!("{:?}",op)
                    };
                    add_operator_support_check(
                        data,
                        op_trait_name,
                    );
                    let op_str = match op {
                        Operator::And => " AND ",
                        Operator::Or => " OR ",
                        Operator::Add => " + ",
                        Operator::Sub => " - ",
                        Operator::Mul => " * ",
                        Operator::Div => " / ",
                        Operator::Mod => " % ",
                        Operator::Concat => " || ",
                        Operator::JsonExtract => " -> ",
                        Operator::JsonExtractText => " ->> ",
                        Operator::BitAnd => " & ",
                        Operator::BitOr => " | ",
                        Operator::BitShiftLeft => " << ",
                        Operator::BitShiftRight => " >> ",
                        Operator::Equal => " = ",
                        Operator::NotEqual => " != ",
                        Operator::GreaterThan => " > ",
                        Operator::GreaterThanOrEqual => " >= ",
                        Operator::LessThan => " < ",
                        Operator::LessThanOrEqual => " <= ",
                        Operator::Like => " LIKE ",
                    };
                    result.push_str(op_str);
                    result.push_str(&not_chain.into_query_string());
                    result.push_str(&expr.into_query_string(
                        data,
                        false,
                        for_custom_select,
                    ));
                }

                result
            }
            Expr::In(val, values) => {
                add_operator_support_check(
                    data,
                    "In",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                match *values {
                    ValueIn::Multiple(vals) => {
                        let mut in_items = Vec::new();
                        for v in vals.into_iter() {
                            in_items.push(v.into_query_string(
                                data,
                                false,
                                for_custom_select,
                            ));
                        }
                        format!("{} IN ({})", val_sql, in_items.join(", "))
                    }
                    ValueIn::SingleColumn(col) => {
                        // Single column reference - convert to Value and process
                        let col_value = Value::Column(col.clone());
                        let col_sql = col_value.into_query_string(
                            data,
                            false,
                            for_custom_select,
                        );
                        format!("{} IN ({})", val_sql, col_sql)
                    }
                    ValueIn::SingleVar(v) => {
                        // Generate dynamic placeholder list based on the runtime length of the collection
                        let debug_str = format!(
                            "Failed to bind items from `{}` to query parameters",
                            v.to_token_stream()
                        );

                        let param_start = *data.current_param_n;

                        // Create a runtime binding and placeholder generation for the collection
                        data.binds.push(quote::quote_spanned! {v.span()=>
                            #[allow(unused_parens)]
                            for __easy_sql_in_item in (#v) {
                                _easy_sql_args.add(__easy_sql_in_item).map_err(anyhow::Error::from_boxed).context(#debug_str)?;
                            }
                        });

                        let format_param_n = data.format_params.len();

                        let before_param_n_name =
                            format_ident!("__easy_sql_before_param_n_{}", format_param_n);
                        let before_param_n = &mut data.before_param_n;

                        data.before_format.push(quote! {
                            let #before_param_n_name:usize;
                        });

                        let parameter_placeholder_call =
                            data.driver.parameter_placeholder_fn(data.sql_crate, v.span());

                        // Create format parameter that generates placeholders at runtime
                        data.format_params.push(quote::quote_spanned! {v.span()=>
                            
                            {
                                #[allow(clippy::needless_borrow)]
                                {
                                    #before_param_n_name = (#v).len();
                                    let mut __easy_sql_in_placeholders = Vec::with_capacity(#before_param_n_name);
                                    for __easy_sql_in_i in 0..#before_param_n_name {
                                        __easy_sql_in_placeholders.push(
                                            #parameter_placeholder_call(#before_param_n #param_start + __easy_sql_in_i)
                                        );
                                    }
                                    __easy_sql_in_placeholders.join(", ")
                                }
                            }
                        });


                        **before_param_n = quote! {#before_param_n_name + #before_param_n};

                        format!("{} IN ({{}})", val_sql)
                    }
                }
            }
            Expr::Between(val, min, max) => {
                add_operator_support_check(
                    data,
                    "Between",
                );
                let val_sql = val.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                let min_sql = min.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                let max_sql = max.into_query_string(
                    data,
                    false,
                    for_custom_select,
                );
                format!("({} BETWEEN {} AND {})", val_sql, min_sql, max_sql)
            }
        }
    }

    /// Check if this expression contains any outside variables ({arg0}, {arg1}, etc.)
    /// Returns a set of argument indices found in the expression.
    pub fn collect_indices_impl(&self, indices: &mut std::collections::BTreeSet<usize>) {
        match self {
            Expr::Value(v) => v.collect_indices_impl(indices),
            Expr::Parenthesized(inner) => inner.collect_indices_impl(indices),
            Expr::OperatorChain(_, first, chain) => {
                first.collect_indices_impl(indices);
                for (_, _, expr) in chain.iter() {
                    expr.collect_indices_impl(indices);
                }
            }
            Expr::IsNull(v) | Expr::IsNotNull(v) => v.collect_indices_impl(indices),
            Expr::In(v, value_in) => {
                v.collect_indices_impl(indices);
                match &**value_in {
                    ValueIn::SingleVar(_) | ValueIn::SingleColumn(_) => {}
                    ValueIn::Multiple(exprs) => {
                        for e in exprs.iter() {
                            e.collect_indices_impl(indices);
                        }
                    }
                }
            }
            Expr::Between(v, low, high) => {
                v.collect_indices_impl(indices);
                low.collect_indices_impl(indices);
                high.collect_indices_impl(indices);
            }
        }
    }
}
