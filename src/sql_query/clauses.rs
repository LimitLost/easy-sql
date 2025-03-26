use serde::{Deserialize, Serialize};

use super::SqlValueRef;
#[derive(Debug, Serialize)]
pub enum WhereExpr<'a> {
    Column(String),
    ValueRef(SqlValueRef<'a>),
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
    And(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    Or(Box<WhereExpr<'a>>, Box<WhereExpr<'a>>),
    Not(Box<WhereExpr<'a>>),
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBy {
    column: String,
    descending: bool,
}
#[derive(Debug, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TableJoin {
    table: String,
    join_type: JoinType,
    alias: Option<String>,
    on: Option<(String, String)>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SelectClauses<'a> {
    where_: Option<WhereClause<'a>>,
    group_by: Option<GroupByClause>,
    having: Option<HavingClause<'a>>,
    order_by: Option<OrderByClause>,
    limit: Option<LimitClause>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct WhereClause<'a> {
    conditions: WhereExpr<'a>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct OrderByClause {
    order_by: Vec<OrderBy>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupByClause {
    columns: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct HavingClause<'a> {
    conditions: WhereExpr<'a>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct LimitClause {
    limit: usize,
}
