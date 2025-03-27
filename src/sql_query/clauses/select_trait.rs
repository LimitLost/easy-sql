use easy_macros::macros::always_context;

use super::SelectClauses;
use super::WhereClause;

#[always_context]
pub trait CanBeSelectClause<'a> {
    fn into_select_clauses(self) -> SelectClauses<'a>;
}

#[always_context]
impl<'a> CanBeSelectClause<'a> for SelectClauses<'a> {
    fn into_select_clauses(self) -> SelectClauses<'a> {
        self
    }
}

#[always_context]
impl<'a> CanBeSelectClause<'a> for WhereClause<'a> {
    fn into_select_clauses(self) -> SelectClauses<'a> {
        SelectClauses {
            distinct: false,
            where_: Some(self),
            group_by: None,
            having: None,
            order_by: None,
            limit: None,
        }
    }
}
