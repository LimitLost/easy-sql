use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::{SqlExpr, SqlValueMaybeRef};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhereClause<'a> {
    pub conditions: SqlExpr<'a>,
}

#[always_context]
impl<'a> WhereClause<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
    ) -> String {
        format!(
            "WHERE {}",
            self.conditions
                .to_query_data(current_binding_n, bindings_list)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HavingClause<'a> {
    pub conditions: SqlExpr<'a>,
}

#[always_context]
impl<'a> HavingClause<'a> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
    ) -> String {
        format!(
            "HAVING {}",
            self.conditions
                .to_query_data(current_binding_n, bindings_list)
        )
    }
}
