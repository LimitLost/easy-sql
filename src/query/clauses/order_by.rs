use easy_macros::always_context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderBy {
    pub column: String,
    pub descending: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderByClause {
    pub order_by: Vec<OrderBy>,
}

#[always_context]
impl OrderByClause {
    pub fn to_query_data(&self) -> String {
        let mut order_by_str = "ORDER BY ".to_string();
        for order in self.order_by.iter() {
            order_by_str.push_str(&order.column);
            if order.descending {
                order_by_str.push_str(" DESC");
            }
            order_by_str.push(',');
        }
        //Removes last comma
        order_by_str.pop();
        order_by_str
    }
}
