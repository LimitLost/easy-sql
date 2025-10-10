use anyhow::Context;
use easy_macros::macros::always_context;

use crate::{
    CanBeSelectClause, Driver, DriverArguments, DriverConnection, DriverRow, Sql, SqlExpr,
    TableJoin, WhereClause,
    easy_executor::{Break, EasyExecutor},
};

use super::{SqlInsert, SqlOutput, SqlUpdate, ToConvert};

// THIS SHIT IS UNSTABLE, AAAAAAA
// pub type Clauses<T,'a> = Option<(fn(T), impl CanBeSelectClause<'a> + Send + Sync);
// pub type WhereClause<T,'a> = (fn(T), impl CanBeSelectClause<'a> + Send + Sync);

#[always_context]
pub trait SqlTable<D: Driver>: Sized
where
    DriverConnection<D>: Send + Sync,
{
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    fn table_joins() -> Vec<TableJoin<'static, D>>;
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    async fn get<'a, Y: ToConvert<D> + Send + Sync, T: SqlOutput<Self, D, Y>>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a, D> + Send + Sync)>,
    ) -> anyhow::Result<T>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        let clauses = clauses.map(|(_, clauses)| clauses);
        // Static table joins can be safely used for any lifetime 'a
        // since they only contain static data
        let joins: Vec<TableJoin<'a, D>> = Self::table_joins()
            .into_iter()
            .map(|j| TableJoin {
                table: j.table,
                join_type: j.join_type,
                alias: j.alias,
                on: j.on.map(|on| unsafe {
                    // SAFETY: table_joins() returns TableJoin<'static, D> which means
                    // the SqlExpr inside only references static data. We can safely
                    // extend its lifetime to 'a since 'static outlives any 'a.
                    std::mem::transmute::<SqlExpr<'static, D>, SqlExpr<'a, D>>(on)
                }),
            })
            .collect();
        let sql = Sql::Select {
            table: Self::table_name(),
            joins,
            clauses: clauses.into_select_clauses(),
        };
        conn.query(&sql).await
    }

    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    /// # How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn get_lazy<'a, T: SqlOutput<Self, D, DriverRow<D>>>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a, D> + Send + Sync)>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
        D: 'a,
    {
        let clauses = clauses.map(|(_, clauses)| clauses);
        // Static table joins can be safely used for any lifetime 'a
        // since they only contain static data
        let joins: Vec<TableJoin<'a, D>> = Self::table_joins()
            .into_iter()
            .map(|j| TableJoin {
                table: j.table,
                join_type: j.join_type,
                alias: j.alias,
                on: j.on.map(|on| unsafe {
                    // SAFETY: table_joins() returns TableJoin<'static, D> which means
                    // the SqlExpr inside only references static data. We can safely
                    // extend its lifetime to 'a since 'static outlives any 'a.
                    std::mem::transmute::<SqlExpr<'static, D>, SqlExpr<'a, D>>(on)
                }),
            })
            .collect();
        let sql = Sql::Select {
            table: Self::table_name(),
            joins,
            clauses: clauses.into_select_clauses(),
        };
        conn.fetch_lazy(&sql, perform).await
    }

    /// Alias for get
    ///
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    async fn select<'a, Y: ToConvert<D> + Send + Sync, T: SqlOutput<Self, D, Y>>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a, D> + Send + Sync)>,
    ) -> anyhow::Result<T>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        Self::get(conn, clauses).await
    }
    /// Alias for get
    ///
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    /// # How to Async inside of closure
    /// (tokio example)
    /// ```rust
    /// //Outside of closure
    /// let handle = tokio::runtime::Handle::current();
    /// //Inside of closure
    /// handle.block_on(async { ... } )
    /// ```
    async fn select_lazy<'a, T: SqlOutput<Self, D, DriverRow<D>>>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a, D> + Send + Sync)>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
        D: 'a,
    {
        Self::get_lazy(conn, clauses, perform).await
    }

    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    async fn exists<'a>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: Option<(fn(Self), impl CanBeSelectClause<'a, D> + Send + Sync)>,
    ) -> anyhow::Result<bool>
    where
        DriverRow<D>: ToConvert<D>,
        bool: SqlOutput<Self, D, DriverRow<D>>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        let clauses = clauses.map(|(_, clauses)| clauses);

        let sql = Sql::Exists {
            table: Self::table_name(),
            joins: vec![],
            clauses: clauses.into_select_clauses(),
        };
        conn.query::<DriverRow<D>, Self, bool>(&sql).await
    }

    async fn insert<'a, I: SqlInsert<Self, D> + Send + Sync>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        to_insert: &'a I,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + SqlOutput<Self, D, ()>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values: to_insert.insert_values()?,
        };

        conn.query::<(), Self, ()>(&sql).await
    }

    async fn insert_returning<
        'a,
        Y: ToConvert<D> + Send + Sync,
        T: SqlOutput<Self, D, Y>,
        I: SqlInsert<Self, D> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        to_insert: &'a I,
    ) -> anyhow::Result<T>
    where
        (): ToConvert<D> + SqlOutput<Self, D, ()>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
    {
        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values: to_insert.insert_values()?,
        };

        conn.query(&sql).await
    }

    async fn insert_returning_lazy<
        'a,
        T: SqlOutput<Self, D, DriverRow<D>>,
        I: SqlInsert<Self, D> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        to_insert: &'a I,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values: to_insert.insert_values()?,
        };

        conn.fetch_lazy(&sql, perform).await
    }

    /// Use `sql_where!` macro to generate the where clause
    async fn delete<'a>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        where_: Option<(fn(Self), WhereClause<'a, D>)>,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + SqlOutput<Self, D, ()>,
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let where_ = where_.map(|(_, where_)| where_);
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };

        conn.query::<(), Self, ()>(&sql).await
    }
    /// Use `sql_where!` macro to generate the where clause
    async fn delete_returning<'a, Y: ToConvert<D> + Send + Sync, T: SqlOutput<Self, D, Y>>(
        conn: &'a mut (impl EasyExecutor<D> + Send + Sync),
        where_: Option<(fn(Self), WhereClause<'a, D>)>,
    ) -> anyhow::Result<T>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let where_ = where_.map(|(_, where_)| where_);
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };

        conn.query(&sql).await
    }

    async fn delete_returning_lazy<
        'a,
        Y: ToConvert<D> + Send + Sync,
        T: SqlOutput<Self, D, DriverRow<D>>,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        where_: Option<WhereClause<'a, D>>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };
        conn.fetch_lazy(&sql, perform).await
    }

    /// Use `sql_where!` macro to generate the `check` and `where` clause
    async fn update<'a>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        mut update: impl SqlUpdate<Self, D> + Send + Sync + 'a,
        where_: Option<(fn(Self), WhereClause<'a, D>)>,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + SqlOutput<Self, D, ()>,
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let where_ = where_.map(|(_, where_)| where_);
        let set = update
            .updates()?
            .into_iter()
            .map(|(col, expr)| {
                // SAFETY: The SqlExpr references data that lives for the duration of this function call.
                // We extend its lifetime to 'a since the SQL execution will complete within this function.
                let expr = unsafe { std::mem::transmute::<SqlExpr<'_, D>, SqlExpr<'a, D>>(expr) };
                (col, expr)
            })
            .collect();
        let sql = Sql::Update {
            table: Self::table_name(),
            set,
            where_,
        };

        conn.query::<(), Self, ()>(&sql).await
    }
    /// Use `sql_where!` macro to generate the where clause
    async fn update_returning<'a, Y: ToConvert<D> + Send + Sync, T: SqlOutput<Self, D, Y>>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        update: &'a mut (impl SqlUpdate<Self, D> + Send + Sync),
        where_: Option<(fn(Self), WhereClause<'a, D>)>,
    ) -> anyhow::Result<T>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let where_ = where_.map(|(_, where_)| where_);
        let set = update
            .updates()?
            .into_iter()
            .map(|(col, expr)| {
                // SAFETY: The SqlExpr references data that lives for the duration of this function call.
                // We extend its lifetime to 'a since the SQL execution will complete within this function.
                let expr = unsafe { std::mem::transmute::<SqlExpr<'_, D>, SqlExpr<'a, D>>(expr) };
                (col, expr)
            })
            .collect();
        let sql = Sql::Update {
            table: Self::table_name(),
            set,
            where_,
        };

        conn.query(&sql).await
    }

    async fn update_returning_lazy<
        'a,
        Y: ToConvert<D> + Send + Sync,
        T: SqlOutput<Self, D, DriverRow<D>>,
        U: SqlUpdate<Self, D> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        update: &'a mut U,
        where_: Option<WhereClause<'a, D>>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver>,
    {
        let set = update
            .updates()?
            .into_iter()
            .map(|(col, expr)| {
                // SAFETY: The SqlExpr references data that lives for the duration of this function call.
                // We extend its lifetime to 'a since the SQL execution will complete within this function.
                let expr = unsafe { std::mem::transmute::<SqlExpr<'_, D>, SqlExpr<'a, D>>(expr) };
                (col, expr)
            })
            .collect();
        let sql = Sql::Update {
            table: Self::table_name(),
            set,
            where_,
        };
        conn.fetch_lazy(&sql, perform).await
    }
}
