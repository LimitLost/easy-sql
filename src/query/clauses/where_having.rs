use easy_macros::always_context;
use serde::{Deserialize, Serialize};

use crate::{Driver, Expr};

#[derive(Debug, Serialize, Deserialize)]
pub struct WhereClause {
    pub conditions: Expr,
}

#[always_context]
impl WhereClause {
    pub fn to_query_data<D: Driver>(&self, current_binding_n: &mut usize) -> String {
        format!(
            "WHERE {}",
            self.conditions.to_query_data::<D>(current_binding_n)
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HavingClause {
    pub conditions: Expr,
}

#[always_context]
impl HavingClause {
    pub fn to_query_data<D: Driver>(&self, current_binding_n: &mut usize) -> String {
        format!(
            "HAVING {}",
            self.conditions.to_query_data::<D>(current_binding_n)
        )
    }
}
