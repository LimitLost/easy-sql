use easy_macros::always_context;
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
pub enum Expr {
    Error,
    Column(String),
    ColumnFromTable { table: String, column: String },
    Value,
    Eq(Box<Expr>, Box<Expr>),
    NotEq(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    GtEq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    LtEq(Box<Expr>, Box<Expr>),
    Like(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Vec<Expr>),
    Between(Box<Expr>, Box<Expr>, Box<Expr>),
    IsNull(Box<Expr>),
    IsNotNull(Box<Expr>),
    OperatorChain(Box<Expr>, Vec<(Operator, Expr)>),
    Not(Box<Expr>),
    Parenthesized(Box<Expr>),
}

#[always_context]
impl Expr {
    pub fn to_query_data<D: Driver>(&self, current_binding_n: &mut usize) -> String {
        match self {
            Expr::Column(s) => {
                let delimeter = D::identifier_delimiter();
                format!("{delimeter}{s}{delimeter}")
            }
            Expr::Value => {
                let current_value_n = *current_binding_n;
                *current_binding_n += 1;
                D::parameter_placeholder(current_value_n)
            }
            Expr::Eq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} = {})", left, right)
            }
            Expr::NotEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} != {})", left, right)
            }
            Expr::Gt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} > {})", left, right)
            }
            Expr::GtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} >= {})", left, right)
            }
            Expr::Lt(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} < {})", left, right)
            }
            Expr::LtEq(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} <= {})", left, right)
            }
            Expr::Like(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1.to_query_data::<D>(current_binding_n);
                format!("({} LIKE {})", left, right)
            }
            Expr::In(where_expr, where_expr1) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let right = where_expr1
                    .iter()
                    .map(|expr| expr.to_query_data::<D>(current_binding_n))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({} IN ({}))", left, right)
            }
            Expr::Between(where_expr, where_expr1, where_expr2) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                let middle = where_expr1.to_query_data::<D>(current_binding_n);
                let right = where_expr2.to_query_data::<D>(current_binding_n);
                format!("({} BETWEEN {} AND {})", left, middle, right)
            }
            Expr::IsNull(where_expr) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                format!("({} IS NULL)", left)
            }
            Expr::IsNotNull(where_expr) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                format!("({} IS NOT NULL)", left)
            }
            Expr::OperatorChain(first_expr, where_exprs) => {
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
            Expr::Not(where_expr) => {
                let left = where_expr.to_query_data::<D>(current_binding_n);
                format!("(NOT {})", left)
            }
            Expr::Parenthesized(where_expr) => {
                format!("({})", where_expr.to_query_data::<D>(current_binding_n))
            }
            Expr::ColumnFromTable { table, column } => {
                let delimeter = D::identifier_delimiter();

                format!("{delimeter}{table}{delimeter}.{delimeter}{column}{delimeter}")
            }
            Expr::Error => {
                panic!("Error in Expr")
            }
        }
    }
}
