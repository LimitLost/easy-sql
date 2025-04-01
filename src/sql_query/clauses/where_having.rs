use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::SqlValueMaybeRef;

pub enum AndOr {
    And,
    Or,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WhereExpr<'a> {
    Column(String),
    ColumnFromTable { table: String, column: String },
    Value(SqlValueMaybeRef<'a>),
    Eq(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    NotEq(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    Gt(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    GtEq(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    Lt(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    LtEq(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    Like(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    In(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    Between(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    IsNull(Box<WhereExpr<'a>>),
    IsNotNull(Box<WhereExpr<'a>>),
    AndOr(Box<WhereExpr<'a>>, Vec<(AndOr, WhereExpr<'a>)>),
    Not(Box<WhereExpr<'a>>),
    Parenthesized(Box<WhereExpr<'a>>),
}

#[always_context]
impl<'a> WhereExpr<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
    ) -> String {
        match self {
            WhereExpr::Column(s) => {
                format!("`{}`", s)
            }
            WhereExpr::Value(sql_value_maybe_ref) => {
                let current_value_n = *current_binding_n;
                *current_binding_n += 1;
                bindings_list.push(sql_value_maybe_ref);
                format!("${}", current_value_n)
            }
            WhereExpr::Eq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} = {})", left, right)
            }
            WhereExpr::NotEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} != {})", left, right)
            }
            WhereExpr::Gt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} > {})", left, right)
            }
            WhereExpr::GtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} >= {})", left, right)
            }
            WhereExpr::Lt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} < {})", left, right)
            }
            WhereExpr::LtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} <= {})", left, right)
            }
            WhereExpr::Like(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} LIKE {})", left, right)
            }
            WhereExpr::In(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} IN {})", left, right)
            }
            WhereExpr::Between(where_expr, where_expr1, where_expr2) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let middle = where_expr1.to_query_data(current_binding_n, bindings_list);
                let right = where_expr2.to_query_data(current_binding_n, bindings_list);
                format!("({} BETWEEN {} AND {})", left, middle, right)
            }
            WhereExpr::IsNull(where_expr) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                format!("({} IS NULL)", left)
            }
            WhereExpr::IsNotNull(where_expr) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                format!("({} IS NOT NULL)", left)
            }
            WhereExpr::And(where_exprs) => {
                let mut iter = where_exprs.iter();

                let mut formatted = String::new();
                if let Some(first) = iter.next() {
                    formatted.push('(');

                    formatted.push_str(&first.to_query_data(current_binding_n, bindings_list));

                    for where_expr in iter {
                        formatted.push_str(" AND ");
                        formatted
                            .push_str(&where_expr.to_query_data(current_binding_n, bindings_list));
                    }

                    formatted.push(')');
                }
                formatted
            }
            WhereExpr::Or(where_exprs) => {
                let mut iter = where_exprs.iter();

                let mut formatted = String::new();
                if let Some(first) = iter.next() {
                    formatted.push('(');

                    formatted.push_str(&first.to_query_data(current_binding_n, bindings_list));

                    for where_expr in iter {
                        formatted.push_str(" OR ");
                        formatted
                            .push_str(&where_expr.to_query_data(current_binding_n, bindings_list));
                    }

                    formatted.push(')');
                }
                formatted
            }
            WhereExpr::Not(where_expr) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                format!("(NOT {})", left)
            }
            WhereExpr::Parenthesized(where_expr) => {
                format!(
                    "({})",
                    where_expr.to_query_data(current_binding_n, bindings_list)
                )
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WhereClause<'a> {
    conditions: WhereExpr<'a>,
}

#[always_context]
impl<'a> WhereClause<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
    ) -> String {
        format!(
            "WHERE {}",
            self.conditions
                .to_query_data(current_binding_n, bindings_list)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HavingClause<'a> {
    conditions: WhereExpr<'a>,
}

#[always_context]
impl<'a> HavingClause<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
    ) -> String {
        format!(
            "HAVING {}",
            self.conditions
                .to_query_data(current_binding_n, bindings_list)
        )
    }
}
