mod clauses;
mod column;
mod sql_type;
mod sql_value;
mod table_field;

use anyhow::Context;
use clauses::{SelectClauses, TableJoin, WhereClause};
pub use column::*;
use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};
pub use sql_type::*;
pub use sql_value::*;

use crate::{Db, SqlOutput};
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Sql<'a> {
    Select {
        distinct: bool,
        // We don't provide output columns here, they are provided inside of SqlOutput trait
        table: String,
        joins: Vec<TableJoin>,
        clauses: SelectClauses<'a>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Vec<SqlValueRef<'a>>>,
    },
    Update {
        table: String,
        set: Vec<(String, SqlValueRef<'a>)>,
        where_: Option<WhereClause<'a>>,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
    Delete {
        table: String,
        where_: Option<WhereClause<'a>>,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
}

type QueryTy<'a> = sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>;

pub(crate) struct QueryData<'a> {
    query: String,
    bindings: Vec<&'a SqlValueRef>,
}

#[always_context]
impl<'a> QueryData<'a> {
    pub fn sqlx(&'a self) -> QueryTy<'a> {
        let mut query = sqlx::query(&self.query);
        for binding in &self.bindings {
            query = query.bind(binding);
        }
        query
    }
}

fn single_value_str(columns_len: usize, current_value_n: &mut usize) -> String {
    let mut single_value_str = String::new();
    for _ in 0..columns_len {
        single_value_str.push('$');
        single_value_str.push_str(&current_value_n.to_string());
        single_value_str.push(',');
    }
    //Removes last comma
    single_value_str.pop();
    single_value_str = format!("({}),", single_value_str);

    single_value_str
}

#[always_context]
impl Sql {
    pub fn query(&self) -> anyhow::Result<QueryData<'_>> {
        Ok(match self {
            Sql::Select { .. } => {
                anyhow::bail!("Select query, but no output expected | self: {:?}", self)
            }
            Sql::Insert {
                table,
                columns,
                values,
            } => {
                let values_str = {
                    let mut current_value_n = 1;
                    let columns_len = columns.len();
                    let mut values_str = String::new();

                    for _ in 0..values.len() {
                        values_str.push_str(&single_value_str(columns_len, &mut current_value_n));
                        values_str.push(',');
                    }
                    //Removes last comma
                    values_str.pop();

                    values_str
                };

                let query_str = format!(
                    "INSERT INTO `{table}` (`{}`) VALUES {values_str}",
                    columns.join("`, `")
                );

                QueryData {
                    query: query_str,
                    bindings: values.iter().flatten().collect(),
                }
            }
            Sql::Update {} => todo!(),
            Sql::Delete {} => todo!(),
        })
    }

    pub fn query_output<'a>(
        &self,
        requested_columns: Vec<RequestedColumn>,
    ) -> anyhow::Result<QueryData<'a>> {
        todo!()
    }
}
