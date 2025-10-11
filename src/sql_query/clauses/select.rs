use crate::Driver;

use super::{GroupByClause, HavingClause, LimitClause, OrderByClause, SqlExpr, WhereClause};
use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Cross,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TableJoin<'a, D: Driver> {
    pub table: &'static str,
    pub join_type: JoinType,
    pub alias: Option<String>,
    pub on: Option<SqlExpr<'a, D>>,
}

#[always_context]
impl<'a, D: Driver> TableJoin<'a, D> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a D::Value<'a>>,
    ) -> String {
        let join_type_str = match self.join_type {
            JoinType::Inner => "INNER",
            JoinType::Left => "LEFT",
            JoinType::Right => "RIGHT",
            JoinType::Cross => "CROSS",
        };
        let delimeter = D::identifier_delimiter();
        let mut join_str = format!(
            "{} JOIN {delimeter}{}{delimeter}",
            join_type_str, self.table
        );
        if let Some(alias) = &self.alias {
            join_str.push_str(&format!(" AS {delimeter}{}{delimeter}", alias));
        }
        if let Some(expr) = &self.on {
            join_str.push_str(&format!(
                " ON {}",
                expr.to_query_data(current_binding_n, bindings_list)
            ));
        }
        join_str
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectClauses<'a, D: Driver> {
    pub distinct: bool,
    pub where_: Option<WhereClause<'a, D>>,
    pub group_by: Option<GroupByClause>,
    pub having: Option<HavingClause<'a, D>>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<LimitClause>,
}

#[always_context]
impl<'a, D: Driver> SelectClauses<'a, D> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a D::Value<'a>>,
    ) -> String {
        let where_str = if let Some(w) = &self.where_ {
            w.to_query_data(current_binding_n, bindings_list)
        } else {
            String::new()
        };
        let group_by_str = if let Some(w) = &self.group_by {
            w.to_query_data()
        } else {
            String::new()
        };

        let having_str = if let Some(h) = &self.having {
            h.to_query_data(current_binding_n, bindings_list)
        } else {
            String::new()
        };
        let order_by_str = if let Some(o) = &self.order_by {
            o.to_query_data()
        } else {
            String::new()
        };
        let limit_str = if let Some(l) = &self.limit {
            l.to_query_data()
        } else {
            String::new()
        };

        format!(
            "{} {} {} {} {}",
            where_str, group_by_str, having_str, order_by_str, limit_str
        )
    }
}
