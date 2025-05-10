use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitClause {
    pub limit: usize,
}

#[always_context]
impl LimitClause {
    pub fn to_query_data(&self) -> String {
        format!("LIMIT {}", self.limit)
    }
}
