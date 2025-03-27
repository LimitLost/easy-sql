use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::SqlValueMaybeRef;

use super::{GroupByClause, HavingClause, LimitClause, OrderByClause, WhereClause};

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

#[always_context]
impl TableJoin {
    pub fn to_query_data(&self) -> String {
        let join_type_str = match self.join_type {
            JoinType::Inner => "INNER",
            JoinType::Left => "LEFT",
            JoinType::Right => "RIGHT",
            JoinType::Full => "FULL",
        };
        let mut join_str = format!("{} JOIN `{}`", join_type_str, self.table);
        if let Some(alias) = &self.alias {
            join_str.push_str(&format!(" AS `{}`", alias));
        }
        if let Some((left, right)) = &self.on {
            join_str.push_str(&format!(
                " ON `{}`.`{}` = `{}`.`{}`",
                self.table, left, self.table, right
            ));
        }
        join_str
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SelectClauses<'a> {
    pub distinct: bool,
    pub where_: Option<WhereClause<'a>>,
    pub group_by: Option<GroupByClause>,
    pub having: Option<HavingClause<'a>>,
    pub order_by: Option<OrderByClause>,
    pub limit: Option<LimitClause>,
}

#[always_context]
impl<'a> SelectClauses<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
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
