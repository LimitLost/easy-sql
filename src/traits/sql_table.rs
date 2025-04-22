use anyhow::Context;
use async_trait::async_trait;
use easy_macros::macros::always_context;

use crate::{
    CanBeSelectClause, Row, Sql, WhereClause,
    easy_executor::{Break, EasyExecutor},
};

use super::{SqlInsert, SqlOutput, SqlUpdate, ToConvert};

#[always_context]
#[async_trait]
pub trait SqlTable: Sized {
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function)
    async fn get<'a, Y: ToConvert + Send + Sync, T: SqlOutput<Self, Y>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a> + Send + Sync)>,
    ) -> anyhow::Result<T> {
        let clauses = clauses.map(|(_, clauses)| clauses);
        let sql = Sql::Select {
            table: Self::table_name(),
            joins: vec![],
            clauses: clauses.into_select_clauses(),
        };
        conn.query(&sql).await
    }

    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function)
    /// # How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn get_lazy<'a, T: SqlOutput<Self, Row>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a> + Send + Sync)>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()> {
        let clauses = clauses.map(|(_, clauses)| clauses);
        let sql = Sql::Select {
            table: Self::table_name(),
            joins: vec![],
            clauses: clauses.into_select_clauses(),
        };
        conn.fetch_lazy(&sql, perform).await
    }

    /// Alias for get
    ///
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function)
    async fn select<'a, Y: ToConvert + Send + Sync, T: SqlOutput<Self, Y>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a> + Send + Sync)>,
    ) -> anyhow::Result<T> {
        Self::get(conn, clauses).await
    }
    /// Alias for get
    ///
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function)
    /// # How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn select_lazy<'a, T: SqlOutput<Self, Row>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a> + Send + Sync)>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()> {
        Self::get_lazy(conn, clauses, perform).await
    }

    async fn insert<I: SqlInsert<Self> + Send + Sync>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        to_insert: &I,
    ) -> anyhow::Result<()> {
        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values: to_insert.insert_values()?,
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
            values: to_insert.insert_values()?,
        };

        conn.query(&sql).await
    }

    //TODO insert returning lazy

    /// Use `sql_where!` macro to generate the where clause
    async fn delete<'a>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        where_: Option<(fn(Self), WhereClause<'a>)>,
    ) -> anyhow::Result<()> {
        let where_ = where_.map(|(_, where_)| where_);
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };

        conn.query::<(), Self, ()>(&sql).await
    }
    /// Use `sql_where!` macro to generate the where clause
    async fn delete_returning<'a, Y: ToConvert + Send + Sync, T: SqlOutput<Self, Y>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        where_: Option<(fn(Self), WhereClause<'a>)>,
    ) -> anyhow::Result<T> {
        let where_ = where_.map(|(_, where_)| where_);
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };

        conn.query(&sql).await
    }

    //TODO delete returning lazy

    /// Use `sql_where!` macro to generate the `check` and `where` clause
    async fn update<'a>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        update: impl SqlUpdate<Self> + Send + Sync,
        where_: Option<(fn(Self), WhereClause<'a>)>,
    ) -> anyhow::Result<()> {
        let where_ = where_.map(|(_, where_)| where_);
        let sql = Sql::Update {
            table: Self::table_name(),
            set: update.updates(),
            where_,
        };

        conn.query::<(), Self, ()>(&sql).await
    }
    /// Use `sql_where!` macro to generate the where clause
    async fn update_returning<'a, Y: ToConvert + Send + Sync, T: SqlOutput<Self, Y>>(
        conn: &mut (impl EasyExecutor + Send + Sync),
        update: impl SqlUpdate<Self> + Send + Sync,
        where_: Option<(fn(Self), WhereClause<'a>)>,
    ) -> anyhow::Result<T> {
        let where_ = where_.map(|(_, where_)| where_);
        let sql = Sql::Update {
            table: Self::table_name(),
            set: update.updates(),
            where_,
        };

        conn.query(&sql).await
    }

    //TODO update returning lazy
}
