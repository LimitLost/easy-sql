use easy_macros::macros::always_context;

use crate::Driver;

use super::SelectClauses;
use super::WhereClause;

#[always_context]
pub trait CanBeSelectClause<'a, D: Driver> {
    fn into_select_clauses(self) -> SelectClauses<'a, D>;
}

#[always_context]
impl<'a, D: Driver> CanBeSelectClause<'a, D> for SelectClauses<'a, D> {
    fn into_select_clauses(self) -> SelectClauses<'a, D> {
        self
    }
}

#[always_context]
impl<'a, D: Driver> CanBeSelectClause<'a, D> for WhereClause<'a, D> {
    fn into_select_clauses(self) -> SelectClauses<'a, D> {
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
#[always_context]
impl<'a, T: CanBeSelectClause<'a, D>, D: Driver> CanBeSelectClause<'a, D> for Option<T> {
    fn into_select_clauses(self) -> SelectClauses<'a, D> {
        match self {
            Some(clauses) => clauses.into_select_clauses(),
            None => SelectClauses {
                distinct: false,
                where_: None,
                group_by: None,
                having: None,
                order_by: None,
                limit: None,
            },
        }
    }
}
