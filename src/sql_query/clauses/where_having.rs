use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::{Driver, SqlExpr};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhereClause<'a, D: Driver> {
    pub conditions: SqlExpr<'a, D>,
}

#[always_context]
impl<'a, D: Driver> WhereClause<'a, D> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a D::Value<'a>>,
    ) -> String {
        format!(
            "WHERE {}",
            self.conditions
                .to_query_data(current_binding_n, bindings_list)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HavingClause<'a, D: Driver> {
    pub conditions: SqlExpr<'a, D>,
}

#[always_context]
impl<'a, D: Driver> HavingClause<'a, D> {
    pub fn to_query_data(
        &'a self,
        current_binding_n: &mut usize,
        bindings_list: &mut Vec<&'a D::Value<'a>>,
    ) -> String {
        format!(
            "HAVING {}",
            self.conditions
                .to_query_data(current_binding_n, bindings_list)
        )
    }
}
