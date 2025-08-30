use std::fmt::Display;

use anyhow::Context;
use easy_macros::macros::always_context;
use serde::{Deserialize, Serialize};

use crate::SqlExpr;

use super::{QueryData, RequestedColumn, SelectClauses, SqlValueMaybeRef, TableJoin, WhereClause};

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

#[derive(Debug, Serialize, Deserialize)]
pub enum Sql<'a> {
    Select {
        // We don't provide output columns here, they are provided inside of SqlOutput trait
        table: &'static str,
        joins: Vec<TableJoin<'a>>,
        clauses: SelectClauses<'a>,
    },
    Exists {
        table: &'static str,
        joins: Vec<TableJoin<'a>>,
        clauses: SelectClauses<'a>,
    },
    Insert {
        table: &'static str,
        columns: Vec<String>,
        values: Vec<Vec<SqlValueMaybeRef<'a>>>,
    },
    Update {
        table: &'static str,
        set: Vec<(String, SqlExpr<'a>)>,
        where_: Option<WhereClause<'a>>,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
    Delete {
        table: &'static str,
        where_: Option<WhereClause<'a>>,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
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
    set: &'a [(String, SqlExpr<'a>)],
    where_: &'a Option<WhereClause>,
    returning: Option<&[RequestedColumn]>,
) -> anyhow::Result<QueryData<'a>> {
    let mut current_binding_n = 1;
    let mut bindings_list = Vec::new();

    let mut set_str = String::new();
    for (column, value) in set {
        let value_parsed = value.to_query_data(&mut current_binding_n, &mut bindings_list);
        set_str.push_str(&format!("`{}` = {},", column, value_parsed));
    }
    //Removes last comma
    set_str.pop();

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
        bindings: bindings_list,
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

fn select_base<'a>(
    joins: &'a [TableJoin],
    table: &'static str,
    clauses: &'a SelectClauses,
    bindings_list: &mut Vec<&'a SqlValueMaybeRef<'a>>,
    requested_str: impl Display,
) -> String {
    let distinct = if clauses.distinct { " DISTINCT" } else { "" };

    let mut current_binding_n = 1;

    let joins_str = {
        let mut joins_str = String::new();
        for join in joins.iter() {
            joins_str.push_str(&join.to_query_data(&mut current_binding_n, bindings_list));
            joins_str.push(' ');
        }
        joins_str
    };

    let clauses_str = clauses.to_query_data(&mut current_binding_n, bindings_list);

    let query_str =
        format!("SELECT{distinct} {requested_str} FROM `{table}` {joins_str} {clauses_str}",);
    query_str
}

#[always_context]
impl Sql<'_> {
    pub(crate) fn query(&self) -> anyhow::Result<QueryData<'_>> {
        Ok(match self {
            Sql::Select { .. } => {
                anyhow::bail!("Select query, but no output expected | self: {:?}", self)
            }
            Sql::Exists {
                table,
                joins,
                clauses,
            } => {
                let mut bindings_list = Vec::new();
                let query_str = format!(
                    "SELECT EXISTS ({})",
                    select_base(joins, table, clauses, &mut bindings_list, "1")
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
            } => insert_query(table, columns, values, None::<&[RequestedColumn]>)?,
            Sql::Update { table, set, where_ } => {
                update_query(table, set, where_, None::<&[RequestedColumn]>)?
            }
            Sql::Delete { table, where_ } => {
                delete_query(table, where_, None::<&[RequestedColumn]>)?
            }
        })
    }

    pub fn query_output(
        &self,
        requested_columns: Vec<RequestedColumn>,
    ) -> anyhow::Result<QueryData<'_>> {
        Ok(match self {
            Sql::Select {
                table,
                joins,
                clauses,
            } => {
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

                let mut bindings_list = Vec::new();

                let query_str =
                    select_base(joins, table, clauses, &mut bindings_list, requested_str);

                QueryData {
                    query: query_str,
                    bindings: bindings_list,
                }
            }
            Sql::Exists {
                table: _,
                joins: _,
                clauses: _,
            } => {
                anyhow::bail!(
                    "Exists query, but no output request expected | self: {:?}",
                    self
                )
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
