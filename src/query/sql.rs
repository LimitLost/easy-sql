use std::fmt::{Debug, Display};

use easy_macros::always_context;

use crate::{Driver, DriverArguments, Expr, QueryBuilder};

use super::{QueryData, RequestedColumn, SelectClauses, TableJoin, WhereClause};

fn single_value_str<D: Driver>(columns_len: usize, current_value_n: &mut usize) -> String {
    let mut single_value_str = String::new();
    for _ in 0..columns_len {
        single_value_str.push_str(&D::parameter_placeholder(*current_value_n));
        single_value_str.push(',');
        *current_value_n += 1;
    }
    //Removes last comma
    single_value_str.pop();
    single_value_str = format!("({}),", single_value_str);

    single_value_str
}
#[derive(Debug)]
pub enum Sql {
    Select {
        // We don't provide output columns here, they are provided inside of Output trait
        table: &'static str,
        joins: Vec<TableJoin>,
        clauses: SelectClauses,
    },
    Exists {
        table: &'static str,
        joins: Vec<TableJoin>,
        clauses: SelectClauses,
    },
    Insert {
        table: &'static str,
        columns: Vec<String>,
        values_count: usize,
    },
    Update {
        table: &'static str,
        set: Vec<(String, Expr)>,
        where_: WhereClause,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
    Delete {
        table: &'static str,
        where_: WhereClause,
        //We don't allow for order by and limit since they are not in Postgres (only Sqlite)
    },
}

#[always_context]
fn insert_query<'a, D: Driver>(
    table: &'static str,
    columns: Vec<String>,
    builder: QueryBuilder<'a, D>,
    values_count: usize,
    returning: Option<Vec<RequestedColumn>>,
) -> QueryData<'a, D>
where
    DriverArguments<'a, D>: sqlx::IntoArguments<'a, D::InternalDriver>,
{
    let args = builder.args();

    let values_str = {
        let mut current_value_n = 0;
        let columns_len = columns.len();
        let mut values_str = String::new();

        for _ in 0..values_count {
            values_str.push_str(&single_value_str::<D>(columns_len, &mut current_value_n));
        }
        //Removes last comma
        values_str.pop();

        values_str
    };

    let returning = if let Some(columns) = returning {
        let mut returning_str = "RETURNING ".to_string();
        for column in columns.into_iter() {
            returning_str.push_str(&column.to_query_data::<D>());
            returning_str.push(',');
        }
        //Removes last comma
        returning_str.pop();
        returning_str
    } else {
        String::new()
    };

    let delimeter = D::identifier_delimiter();

    let query_str = format!(
        "INSERT INTO {delimeter}{table}{delimeter} ({delimeter}{}{delimeter}) VALUES {values_str} {returning}",
        columns.join(&format!("{delimeter}, {delimeter}"))
    );

    QueryData::new(query_str, args)
}

#[always_context]
fn update_query<'a, D: Driver>(
    table: &'static str,
    set: Vec<(String, Expr)>,
    where_: WhereClause,
    returning: Option<Vec<RequestedColumn>>,
    builder: QueryBuilder<'a, D>,
) -> QueryData<'a, D>
where
    DriverArguments<'a, D>: sqlx::IntoArguments<'a, D::InternalDriver>,
{
    let args = builder.args();

    let mut current_binding_n = 0;

    let delimeter = D::identifier_delimiter();

    let mut set_str = String::new();
    for (column, value) in set {
        let value_parsed = value.to_query_data::<D>(&mut current_binding_n);
        set_str.push_str(&format!("{delimeter}{column}{delimeter} = {value_parsed},"));
    }
    //Removes last comma
    set_str.pop();

    let where_str = where_.to_query_data::<D>(&mut current_binding_n);

    let returning = if let Some(columns) = returning {
        let mut returning_str = "RETURNING ".to_string();
        for column in columns.iter() {
            returning_str.push_str(&column.to_query_data::<D>());
            returning_str.push(',');
        }
        //Removes last comma
        returning_str.pop();
        returning_str
    } else {
        String::new()
    };

    let query_str =
        format!("UPDATE {delimeter}{table}{delimeter} SET {set_str} {where_str} {returning}");

    QueryData::new(query_str, args)
}

#[always_context]
fn delete_query<'a, D: Driver>(
    table: &'static str,
    where_: WhereClause,
    returning: Option<Vec<RequestedColumn>>,
    builder: QueryBuilder<'a, D>,
) -> QueryData<'a, D>
where
    DriverArguments<'a, D>: sqlx::IntoArguments<'a, D::InternalDriver>,
{
    let mut current_binding_n = 0;
    let args = builder.args();

    let where_str = where_.to_query_data::<D>(&mut current_binding_n);

    let returning = if let Some(columns) = returning {
        let mut returning_str = "RETURNING ".to_string();
        for column in columns.iter() {
            returning_str.push_str(&column.to_query_data::<D>());
            returning_str.push(',');
        }
        //Removes last comma
        returning_str.pop();
        returning_str
    } else {
        String::new()
    };

    let delimeter = D::identifier_delimiter();

    let query_str = format!("DELETE FROM {delimeter}{table}{delimeter} {where_str} {returning}");

    QueryData::new(query_str, args)
}

fn select_base<D: Driver>(
    joins: Vec<TableJoin>,
    table: &'static str,
    clauses: SelectClauses,
    requested_str: impl Display,
) -> String {
    let distinct = if clauses.distinct { " DISTINCT" } else { "" };

    let mut current_binding_n = 0;

    let joins_str = {
        let mut joins_str = String::new();
        for join in joins.iter() {
            joins_str.push_str(&join.to_query_data::<D>(&mut current_binding_n));
            joins_str.push(' ');
        }
        joins_str
    };

    let clauses_str = clauses.to_query_data::<D>(&mut current_binding_n);

    let delimeter = D::identifier_delimiter();

    let query_str = format!(
        "SELECT{distinct} {requested_str} FROM {delimeter}{table}{delimeter} {joins_str} {clauses_str}"
    );
    query_str
}

#[always_context]
impl Sql {
    pub(crate) fn query<'a, D: Driver>(
        self,
        builder: QueryBuilder<'a, D>,
    ) -> anyhow::Result<QueryData<'a, D>>
    where
        DriverArguments<'a, D>: sqlx::IntoArguments<'a, D::InternalDriver> + Debug,
    {
        Ok(match self {
            Sql::Select { .. } => {
                anyhow::bail!("Select query, but no output expected | self: {:?}", self)
            }
            Sql::Exists {
                table,
                joins,
                clauses,
            } => {
                let query_str = format!(
                    "SELECT EXISTS ({})",
                    select_base::<D>(joins, table, clauses, "1")
                );
                QueryData::new(query_str, builder.args())
            }
            Sql::Insert {
                table,
                columns,
                values_count,
            } => insert_query(table, columns, builder, values_count, None),
            Sql::Update { table, set, where_ } => update_query(table, set, where_, None, builder),
            Sql::Delete { table, where_ } => delete_query(table, where_, None, builder),
        })
    }

    pub fn query_output<'a, D: Driver>(
        self,
        builder: QueryBuilder<'a, D>,
        requested_columns: Vec<RequestedColumn>,
    ) -> anyhow::Result<QueryData<'a, D>>
    where
        DriverArguments<'a, D>: sqlx::IntoArguments<'a, D::InternalDriver> + Debug,
    {
        Ok(match self {
            Sql::Select {
                table,
                joins,
                clauses,
            } => {
                let requested_str = {
                    let mut requested_str = String::new();
                    for column in requested_columns.into_iter() {
                        requested_str.push_str(&column.to_query_data::<D>());
                        requested_str.push(',');
                    }
                    //Removes last comma
                    requested_str.pop();
                    requested_str
                };

                let query_str = select_base::<D>(joins, table, clauses, requested_str);

                QueryData::new(query_str, builder.args())
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
                values_count,
            } => insert_query(
                table,
                columns,
                builder,
                values_count,
                Some(requested_columns),
            ),
            Sql::Update { table, set, where_ } => {
                update_query(table, set, where_, Some(requested_columns), builder)
            }
            Sql::Delete { table, where_ } => {
                delete_query(table, where_, Some(requested_columns), builder)
            }
        })
    }
}
