mod clauses;
mod column;
mod sql_type;
mod sql_value;
mod table_field;

use anyhow::Context;
pub use clauses::*;
pub use column::*;
use easy_macros::{helpers::context, macros::always_context};
use serde::{Deserialize, Serialize};
pub use sql_type::*;
pub use sql_value::*;

use crate::Db;
#[derive(Debug, Serialize, Deserialize)]
pub enum Sql<'a> {
    Select {
        // We don't provide output columns here, they are provided inside of SqlOutput trait
        table: &'static str,
        joins: Vec<TableJoin>,
        clauses: SelectClauses<'a>,
    },
    Insert {
        table: &'static str,
        columns: Vec<String>,
        values: Vec<Vec<SqlValueMaybeRef<'a>>>,
    },
    Update {
        table: &'static str,
        set: Vec<(String, SqlValueMaybeRef<'a>)>,
        where_: Option<WhereClause<'a>>,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
    Delete {
        table: &'static str,
        where_: Option<WhereClause<'a>>,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
}

type QueryTy<'a> = sqlx::query::Query<'a, Db, <Db as sqlx::Database>::Arguments<'a>>;

pub struct QueryData<'a> {
    query: String,
    bindings: Vec<&'a SqlValueMaybeRef<'a>>,
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
        *current_value_n += 1;
    }
    //Removes last comma
    single_value_str.pop();
    single_value_str = format!("({}),", single_value_str);

    single_value_str
}

#[always_context]
fn insert_query<'a>(
    table: &'static str,
    columns: &[String],
    values: &'a Vec<Vec<SqlValueMaybeRef>>,
    returning: Option<&[RequestedColumn]>,
) -> anyhow::Result<QueryData<'a>> {
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

    let returning = if let Some(columns) = returning {
        let mut returning_str = "RETURNING ".to_string();
        for column in columns.iter() {
            returning_str.push_str(&column.to_query_data());
            returning_str.push(',');
        }
        //Removes last comma
        returning_str.pop();
        returning_str
    } else {
        String::new()
    };

    let query_str = format!(
        "INSERT INTO `{table}` (`{}`) VALUES {values_str} {returning}",
        columns.join("`, `")
    );

    Ok(QueryData {
        query: query_str,
        bindings: values.iter().flatten().collect(),
    })
}

#[always_context]
fn update_query<'a>(
    table: &'static str,
    set: &'a [(String, SqlValueMaybeRef<'a>)],
    where_: &'a Option<WhereClause>,
    returning: Option<&[RequestedColumn]>,
) -> anyhow::Result<QueryData<'a>> {
    let mut current_binding_n = 1;
    let mut set_str = String::new();
    for (column, _) in set {
        set_str.push_str(&format!("`{}` = ${},", column, current_binding_n));
        current_binding_n += 1;
    }
    //Removes last comma
    set_str.pop();

    let mut bindings_list = Vec::new();

    let where_str = if let Some(w) = where_ {
        w.to_query_data(&mut current_binding_n, &mut bindings_list)
    } else {
        String::new()
    };

    let returning = if let Some(columns) = returning {
        let mut returning_str = "RETURNING ".to_string();
        for column in columns.iter() {
            returning_str.push_str(&column.to_query_data());
            returning_str.push(',');
        }
        //Removes last comma
        returning_str.pop();
        returning_str
    } else {
        String::new()
    };

    let query_str = format!("UPDATE `{table}` SET {set_str} {where_str} {returning}");

    Ok(QueryData {
        query: query_str,
        bindings: set.iter().map(|(_, v)| v).chain(bindings_list).collect(),
    })
}

#[always_context]
fn delete_query<'a>(
    table: &'static str,
    where_: &'a Option<WhereClause>,
    returning: Option<&[RequestedColumn]>,
) -> anyhow::Result<QueryData<'a>> {
    let mut current_binding_n = 1;
    let mut bindings_list = Vec::new();

    let where_str = if let Some(w) = where_ {
        w.to_query_data(&mut current_binding_n, &mut bindings_list)
    } else {
        String::new()
    };

    let returning = if let Some(columns) = returning {
        let mut returning_str = "RETURNING ".to_string();
        for column in columns.iter() {
            returning_str.push_str(&column.to_query_data());
            returning_str.push(',');
        }
        //Removes last comma
        returning_str.pop();
        returning_str
    } else {
        String::new()
    };

    let query_str = format!("DELETE FROM `{table}` {where_str} {returning}",);

    Ok(QueryData {
        query: query_str,
        bindings: bindings_list,
    })
}

#[always_context]
impl Sql<'_> {
    pub(crate) fn query(&self) -> anyhow::Result<QueryData<'_>> {
        Ok(match self {
            Sql::Select { .. } => {
                anyhow::bail!("Select query, but no output expected | self: {:?}", self)
            }
            Sql::Insert {
                table,
                columns,
                values,
            } => insert_query(table, columns, values, None::<&[RequestedColumn]>)?,
            Sql::Update { table, set, where_ } => {
                update_query(table, set, where_, None::<&[RequestedColumn]>)?
            }
            Sql::Delete { table, where_ } => {
                delete_query(table, where_, None::<&[RequestedColumn]>)?
            }
        })
    }

    pub(crate) fn query_output(
        &self,
        requested_columns: Vec<RequestedColumn>,
    ) -> anyhow::Result<QueryData<'_>> {
        Ok(match self {
            Sql::Select {
                table,
                joins,
                clauses,
            } => {
                let distinct = if clauses.distinct { " DISTINCT" } else { "" };

                let requested_str = {
                    let mut requested_str = String::new();
                    for column in requested_columns.iter() {
                        requested_str.push_str(&column.to_query_data());
                        requested_str.push(',');
                    }
                    //Removes last comma
                    requested_str.pop();
                    requested_str
                };

                let joins_str = {
                    let mut joins_str = String::new();
                    for join in joins.iter() {
                        joins_str.push_str(&join.to_query_data());
                        joins_str.push(' ');
                    }
                    joins_str
                };

                let mut current_binding_n = 1;
                let mut bindings_list = Vec::new();

                let clauses_str = clauses.to_query_data(&mut current_binding_n, &mut bindings_list);

                let query_str = format!(
                    "SELECT{distinct} {requested_str} FROM `{table}` {joins_str} {clauses_str}",
                );

                QueryData {
                    query: query_str,
                    bindings: bindings_list,
                }
            }
            Sql::Insert {
                table,
                columns,
                values,
            } => insert_query(table, columns, values, Some(&requested_columns))?,
            Sql::Update { table, set, where_ } => {
                update_query(table, set, where_, Some(&requested_columns))?
            }
            Sql::Delete { table, where_ } => delete_query(table, where_, Some(&requested_columns))?,
        })
    }
}
