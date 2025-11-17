use easy_macros::always_context;

use super::SelectClauses;
use super::WhereClause;

#[always_context]
pub trait CanBeSelectClause {
    fn into_select_clauses(self) -> SelectClauses;
}

#[always_context]
impl CanBeSelectClause for SelectClauses {
    fn into_select_clauses(self) -> SelectClauses {
        self
    }
}

#[always_context]
impl CanBeSelectClause for WhereClause {
    fn into_select_clauses(self) -> SelectClauses {
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
impl<T: CanBeSelectClause> CanBeSelectClause for Option<T> {
    fn into_select_clauses(self) -> SelectClauses {
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
