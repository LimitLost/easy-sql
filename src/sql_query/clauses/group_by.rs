use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GroupByClause {
    columns: Vec<String>,
}

#[always_context]
impl GroupByClause {
    pub fn to_query_data(&self) -> String {
        let mut group_by_str = "GROUP BY ".to_string();
        for column in self.columns.iter() {
            group_by_str.push_str(column);
            group_by_str.push(',');
        }
        //Removes last comma
        group_by_str.pop();
        group_by_str
    }
}
