use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::Driver;

#[derive(Debug, Serialize, Deserialize)]
pub enum Operator {
    ///AND Keyword
    And,
    ///OR Keyword
    Or,
    ///+
    Add,
    ///-
    Sub,
    ///*
    Mul,
    /// /
    Div,
    ///%
    Mod,
    /// ||
    Concat,
    ///-> or ->>
    JsonExtract,
    /// &
    BitAnd,
    /// |
    BitOr,
    /// <<
    BitShiftLeft,
    /// >>
    BitShiftRight,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SqlExpr<'a, D: Driver> {
    Error,
    Column(String),
    ColumnFromTable {
        table: String,
        column: String,
    },
    Value(D::Value<'a>),
    Eq(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    NotEq(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    Gt(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    GtEq(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    Lt(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    LtEq(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    Like(Box<SqlExpr<'a, D>>, Box<SqlExpr<'a, D>>),
    In(Box<SqlExpr<'a, D>>, Vec<SqlExpr<'a, D>>),
    Between(
        Box<SqlExpr<'a, D>>,
        Box<SqlExpr<'a, D>>,
        Box<SqlExpr<'a, D>>,
    ),
    IsNull(Box<SqlExpr<'a, D>>),
    IsNotNull(Box<SqlExpr<'a, D>>),
    OperatorChain(Box<SqlExpr<'a, D>>, Vec<(Operator, SqlExpr<'a, D>)>),
    Not(Box<SqlExpr<'a, D>>),
    Parenthesized(Box<SqlExpr<'a, D>>),
}

#[always_context]
impl<'a, D: Driver> SqlExpr<'a, D> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a D::Value<'a>>,
    ) -> String {
        match self {
            SqlExpr::Column(s) => {
                format!("`{}`", s)
            }
            SqlExpr::Value(sql_value_maybe_ref) => {
                let current_value_n = *current_binding_n;
                *current_binding_n += 1;
                bindings_list.push(sql_value_maybe_ref);
                format!("${}", current_value_n)
            }
            SqlExpr::Eq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} = {})", left, right)
            }
            SqlExpr::NotEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} != {})", left, right)
            }
            SqlExpr::Gt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} > {})", left, right)
            }
            SqlExpr::GtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} >= {})", left, right)
            }
            SqlExpr::Lt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} < {})", left, right)
            }
            SqlExpr::LtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} <= {})", left, right)
            }
            SqlExpr::Like(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1.to_query_data(current_binding_n, bindings_list);
                format!("({} LIKE {})", left, right)
            }
            SqlExpr::In(where_expr, where_expr1) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let right = where_expr1
                    .iter()
                    .map(|expr| expr.to_query_data(current_binding_n, bindings_list))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({} IN ({}))", left, right)
            }
            SqlExpr::Between(where_expr, where_expr1, where_expr2) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                let middle = where_expr1.to_query_data(current_binding_n, bindings_list);
                let right = where_expr2.to_query_data(current_binding_n, bindings_list);
                format!("({} BETWEEN {} AND {})", left, middle, right)
            }
            SqlExpr::IsNull(where_expr) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                format!("({} IS NULL)", left)
            }
            SqlExpr::IsNotNull(where_expr) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                format!("({} IS NOT NULL)", left)
            }
            SqlExpr::OperatorChain(first_expr, where_exprs) => {
                let mut formatted = String::new();
                formatted.push('(');

                formatted.push_str(&first_expr.to_query_data(current_binding_n, bindings_list));

                for (and_or, where_expr) in where_exprs.iter() {
                    match and_or {
                        Operator::And => formatted.push_str(" AND "),
                        Operator::Or => formatted.push_str(" OR "),
                        Operator::Add => formatted.push_str(" + "),
                        Operator::Sub => formatted.push_str(" - "),
                        Operator::Mul => formatted.push_str(" * "),
                        Operator::Div => formatted.push_str(" / "),
                        Operator::Mod => formatted.push_str(" % "),
                        Operator::Concat => formatted.push_str(" || "),
                        Operator::JsonExtract => formatted.push_str(" -> "),
                        Operator::BitAnd => formatted.push_str(" & "),
                        Operator::BitOr => formatted.push_str(" | "),
                        Operator::BitShiftLeft => formatted.push_str(" << "),
                        Operator::BitShiftRight => formatted.push_str(" >> "),
                    }
                    formatted.push_str(&where_expr.to_query_data(current_binding_n, bindings_list));
                }

                formatted.push(')');

                formatted
            }
            SqlExpr::Not(where_expr) => {
                let left = where_expr.to_query_data(current_binding_n, bindings_list);
                format!("(NOT {})", left)
            }
            SqlExpr::Parenthesized(where_expr) => {
                format!(
                    "({})",
                    where_expr.to_query_data(current_binding_n, bindings_list)
                )
            }
            SqlExpr::ColumnFromTable { table, column } => {
                format!("{}.{}", table, column)
            }
            SqlExpr::Error => {
                panic!("Error in SqlExpr")
            }
        }
    }
}
