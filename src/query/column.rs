use easy_macros::macros::always_context;

use crate::Driver;

#[derive(Debug)]
pub struct RequestedColumn {
    pub table_name: Option<&'static str>,
    pub name: String,
    pub alias: Option<String>,
}

#[always_context]
impl RequestedColumn {
    pub fn to_query_data<D: Driver>(&self) -> String {
        let delimiter = D::identifier_delimiter();
        let table_name = if let Some(table) = self.table_name {
            format!("{delimiter}{table}{delimiter}.")
        } else {
            String::new()
        };
        if let Some(alias) = &self.alias {
            format!(
                "{table_name}{delimiter}{}{delimiter} AS {delimiter}{alias}{delimiter}",
                self.name
            )
        } else {
            format!("{table_name}{delimiter}{}{delimiter}", self.name)
        }
    }
}
