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
pub enum SqlExpr {
    Error,
    Column(String),
    ColumnFromTable { table: String, column: String },
    Value,
    Eq(Box<SqlExpr>, Box<SqlExpr>),
    NotEq(Box<SqlExpr>, Box<SqlExpr>),
    Gt(Box<SqlExpr>, Box<SqlExpr>),
    GtEq(Box<SqlExpr>, Box<SqlExpr>),
    Lt(Box<SqlExpr>, Box<SqlExpr>),
    LtEq(Box<SqlExpr>, Box<SqlExpr>),
    Like(Box<SqlExpr>, Box<SqlExpr>),
    In(Box<SqlExpr>, Vec<SqlExpr>),
    Between(Box<SqlExpr>, Box<SqlExpr>, Box<SqlExpr>),
    IsNull(Box<SqlExpr>),
    IsNotNull(Box<SqlExpr>),
    OperatorChain(Box<SqlExpr>, Vec<(Operator, SqlExpr)>),
    Not(Box<SqlExpr>),
    Parenthesized(Box<SqlExpr>),
}

#[always_context]
impl SqlExpr {
    pub fn to_query_data<D: Driver>(&self, current_binding_n: &mut usize) -> String {
        match self {
            SqlExpr::Column(s) => {
                let delimeter = D::identifier_delimiter();
                format!("{delimeter}{s}{delimeter}")
            }
            SqlExpr::Value => {
                let current_value_n = *current_binding_n;
                *current_binding_n += 1;
                D::parameter_placeholder(current_value_n)
            }
            SqlExpr::Eq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} = {})", left, right)
            }
            SqlExpr::NotEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} != {})", left, right)
            }
            SqlExpr::Gt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} > {})", left, right)
            }
            SqlExpr::GtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} >= {})", left, right)
            }
            SqlExpr::Lt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} < {})", left, right)
            }
            SqlExpr::LtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} <= {})", left, right)
            }
            SqlExpr::Like(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} LIKE {})", left, right)
            }
            SqlExpr::In(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1
                    .iter()
                    .map(|expr| expr.to_query_data::<D>(current_binding_n))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({} IN ({}))", left, right)
            }
            SqlExpr::Between(where_expr, where_expr1, where_expr2) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let middle = where_expr1.to_query_data::<D>(current_binding_n);
                let right = where_expr2.to_query_data::<D>(current_binding_n);
                format!("({} BETWEEN {} AND {})", left, middle, right)
            }
            SqlExpr::IsNull(where_expr) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                format!("({} IS NULL)", left)
            }
            SqlExpr::IsNotNull(where_expr) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                format!("({} IS NOT NULL)", left)
            }
            SqlExpr::OperatorChain(first_expr, where_exprs) => {
                let mut formatted = String::new();
                formatted.push('(');

                formatted.push_str(&first_expr.to_query_data::<D>(current_binding_n));

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
                    formatted.push_str(&where_expr.to_query_data::<D>(current_binding_n));
                }

                formatted.push(')');

                formatted
            }
            SqlExpr::Not(where_expr) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                format!("(NOT {})", left)
            }
            SqlExpr::Parenthesized(where_expr) => {
                format!("({})", where_expr.to_query_data::<D>(current_binding_n))
            }
            SqlExpr::ColumnFromTable { table, column } => {
                let delimeter = D::identifier_delimiter();

                format!("{delimeter}{table}{delimeter}.{delimeter}{column}{delimeter}")
            }
            SqlExpr::Error => {
                panic!("Error in SqlExpr")
            }
        }
    }
}
