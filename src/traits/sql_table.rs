use async_trait::async_trait;
use easy_macros::macros::always_context;

use crate::{CanBeSelectClause, Row, Sql, easy_executor::EasyExecutor};

use super::{SqlInsert, SqlOutput, ToConvert};

#[always_context]
#[async_trait]
pub trait SqlTable: Sized {
    fn table_name() -> &'static str;

    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function)
    async fn get<'a, Y: ToConvert + Send + Sync, T: SqlOutput<Self, Y>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        (_, clauses): (fn(Self), impl CanBeSelectClause<'a> + Send + Sync),
    ) -> anyhow::Result<T> {
        let sql = Sql::Select {
            table: Self::table_name(),
            joins: vec![],
            clauses: clauses.into_select_clauses(),
        };
        conn.query(&sql).await
    }

    /// Alias for get
    ///
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function)
    async fn select<'a, Y: ToConvert + Send + Sync, T: SqlOutput<Self, Y>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        clauses: (fn(Self), impl CanBeSelectClause<'a> + Send + Sync),
    ) -> anyhow::Result<T> {
        Self::get(conn, clauses).await
    }

    async fn insert<I: SqlInsert<Self> + Send + Sync>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        to_insert: &I,
    ) -> anyhow::Result<()> {
        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values: to_insert.insert_values(),
        };

        conn.query::<(), Self, ()>(&sql).await
    }

    async fn insert_returning<
        Y: ToConvert + Send + Sync,
        T: SqlOutput<Self, Y>,
        I: SqlInsert<Self> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor + Send + Sync),
        to_insert: &I,
    ) -> anyhow::Result<T> {
        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values: to_insert.insert_values(),
        };

        conn.query(&sql).await
    }
}
