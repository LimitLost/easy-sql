use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::SqlValueMaybeRef;

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
pub enum SqlExpr<'a> {
    Error,
    Column(String),
    ColumnFromTable { table: String, column: String },
    Value(SqlValueMaybeRef<'a>),
    Eq(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    NotEq(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    Gt(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    GtEq(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    Lt(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    LtEq(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    Like(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    In(Box<SqlExpr<'a>>, Vec<SqlExpr<'a>>),
    Between(Box<SqlExpr<'a>>, Box<SqlExpr<'a>>, Box<SqlExpr<'a>>),
    IsNull(Box<SqlExpr<'a>>),
    IsNotNull(Box<SqlExpr<'a>>),
    OperatorChain(Box<SqlExpr<'a>>, Vec<(Operator, SqlExpr<'a>)>),
    Not(Box<SqlExpr<'a>>),
    Parenthesized(Box<SqlExpr<'a>>),
}

#[always_context]
impl<'a> SqlExpr<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
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
