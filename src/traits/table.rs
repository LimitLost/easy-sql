use anyhow::Context;
use easy_macros::macros::always_context;
use std::fmt::Debug;

use crate::{
    CanBeSelectClause, Driver, DriverArguments, DriverConnection, DriverRow, Sql, TableJoin,
    WhereClause,
    easy_executor::{Break, EasyExecutor},
};

use super::{Insert, Output, Update, ToConvert};
use crate::QueryBuilder;

#[always_context]
pub trait Table<D: Driver>: Sized
where
    DriverConnection<D>: Send + Sync,
{
    fn table_name() -> &'static str;
    fn primary_keys() -> Vec<&'static str>;

    fn table_joins(builder: &mut QueryBuilder<'_, D>) -> Vec<TableJoin>;
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    async fn get<
        'a,
        Y: ToConvert<D> + Send + Sync + 'static,
        T: Output<Self, D, DataToConvert = Y>,
        CO: CanBeSelectClause + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<CO>,
    ) -> anyhow::Result<T>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: Debug,
    {
        let mut builder = QueryBuilder::default();
        let joins: Vec<TableJoin> = Self::table_joins(&mut builder);

        let clauses = clauses(&mut builder)?;

        let sql = Sql::Select {
            table: Self::table_name(),
            joins,
            clauses: clauses.into_select_clauses(),
        };
        conn.query(sql, builder).await
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
    async fn get_lazy<
        'a,
        T: Output<Self, D, DataToConvert = DriverRow<D>>,
        CO: CanBeSelectClause + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<CO>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
        D: 'a,
    {
        let mut builder = QueryBuilder::default();
        let joins: Vec<TableJoin> = Self::table_joins(&mut builder);
        let clauses = clauses(&mut builder)?;
        let sql = Sql::Select {
            table: Self::table_name(),
            joins,
            clauses: clauses.into_select_clauses(),
        };
        conn.fetch_lazy(sql, builder, perform).await
    }

    /// Alias for get
    ///
    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    async fn select<
        'a,
        Y: ToConvert<D> + Send + Sync + 'static,
        T: Output<Self, D, DataToConvert = Y>,
        CO: CanBeSelectClause + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<CO>,
    ) -> anyhow::Result<T>
    where
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: Debug,
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
    async fn select_lazy<
        'a,
        T: Output<Self, D, DataToConvert = DriverRow<D>>,
        CO: CanBeSelectClause + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<CO>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
        D: 'a,
    {
        Self::get_lazy(conn, clauses, perform).await
    }

    /// Use `sql!` or `sql_where!` macros to generate clauses (second argument to this function), or `None` for all rows
    async fn exists<'a, CO: CanBeSelectClause + Send + Sync>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        clauses: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<CO>,
    ) -> anyhow::Result<bool>
    where
        DriverRow<D>: ToConvert<D>,
        bool: Output<Self, D, DataToConvert = DriverRow<D>>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: Debug,
    {
        let mut builder = QueryBuilder::default();

        let joins: Vec<TableJoin> = Self::table_joins(&mut builder);

        let clauses = clauses(&mut builder)?;

        let sql = Sql::Exists {
            table: Self::table_name(),
            joins,
            clauses: clauses.into_select_clauses(),
        };
        conn.query::<DriverRow<D>, Self, bool>(sql, builder).await
    }

    async fn insert<'a, I: Insert<'a, Self, D> + Send + Sync>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        to_insert: I,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + Output<Self, D, DataToConvert = ()>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: Debug,
    {
        let mut builder = QueryBuilder::default();
        let values_count = to_insert.insert_values(&mut builder)?;

        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values_count,
        };

        conn.query::<(), Self, ()>(sql, builder).await
    }

    async fn insert_returning<
        'a,
        Y: ToConvert<D> + Send + Sync + 'static,
        T: Output<Self, D, DataToConvert = Y>,
        I: Insert<'a, Self, D> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        to_insert: I,
    ) -> anyhow::Result<T>
    where
        (): ToConvert<D> + Output<Self, D, DataToConvert = ()>,
        DriverConnection<D>: Send + Sync,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: Debug,
    {
        let mut builder = QueryBuilder::default();
        let values_count = to_insert.insert_values(&mut builder)?;

        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values_count,
        };

        conn.query(sql, builder).await
    }

    async fn insert_returning_lazy<
        'a,
        T: Output<Self, D, DataToConvert = DriverRow<D>>,
        I: Insert<'a, Self, D> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        to_insert: I,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D> + 'static,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
    {
        let mut builder = QueryBuilder::default();
        let values_count = to_insert.insert_values(&mut builder)?;

        let sql = Sql::Insert {
            table: Self::table_name(),
            columns: I::insert_columns(),
            values_count,
        };

        conn.fetch_lazy(sql, builder, perform).await
    }

    /// Use `sql_where!` macro to generate the where clause
    async fn delete<'a>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        where_: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<WhereClause>,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + Output<Self, D, DataToConvert = ()>,
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
    {
        let mut builder = QueryBuilder::default();
        let where_ = where_(&mut builder)?;
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };

        conn.query::<(), Self, ()>(sql, builder).await
    }
    /// Use `sql_where!` macro to generate the where clause
    async fn delete_returning<
        'a,
        Y: ToConvert<D> + Send + Sync + 'static,
        T: Output<Self, D, DataToConvert = Y>,
    >(
        conn: &'a mut (impl EasyExecutor<D> + Send + Sync),
        where_: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<WhereClause>,
    ) -> anyhow::Result<T>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
    {
        let mut builder = QueryBuilder::default();
        let where_ = where_(&mut builder)?;
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };

        conn.query(sql, builder).await
    }

    async fn delete_returning_lazy<
        'a,
        Y: ToConvert<D> + Send + Sync,
        T: Output<Self, D, DataToConvert = DriverRow<D>>,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        where_: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<WhereClause>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D> + 'static,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
    {
        let mut builder = QueryBuilder::default();
        let where_ = where_(&mut builder)?;
        let sql = Sql::Delete {
            table: Self::table_name(),
            where_,
        };
        conn.fetch_lazy(sql, builder, perform).await
    }

    /// Use `sql_where!` macro to generate the `check` and `where` clause
    async fn update<'a>(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        update: impl Update<'a, Self, D> + Send + Sync + 'a,
        where_: impl Fn(&mut QueryBuilder<'a, D>) -> anyhow::Result<WhereClause>,
    ) -> anyhow::Result<()>
    where
        (): ToConvert<D> + Output<Self, D, DataToConvert = ()>,
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
        D: 'a,
    {
        let mut builder = QueryBuilder::default();
        let set = update.updates(&mut builder)?;
        let where_ = where_(&mut builder)?;
        let sql = Sql::Update {
            table: Self::table_name(),
            set,
            where_,
        };

        conn.query::<(), Self, ()>(sql, builder).await
    }
    /// Use `sql_where!` macro to generate the where clause
    async fn update_returning<
        'a,
        Y: ToConvert<D> + Send + Sync + 'static,
        T: Output<Self, D, DataToConvert = Y>,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        update: impl Update<'a, Self, D> + Send + Sync,
        where_: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<WhereClause>,
    ) -> anyhow::Result<T>
    where
        DriverRow<D>: ToConvert<D>,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
    {
        let mut builder = QueryBuilder::default();
        let set = update.updates(&mut builder)?;
        let where_ = where_(&mut builder)?;
        let sql = Sql::Update {
            table: Self::table_name(),
            set,
            where_,
        };

        conn.query(sql, builder).await
    }

    async fn update_returning_lazy<
        'a,
        Y: ToConvert<D> + Send + Sync,
        T: Output<Self, D, DataToConvert = DriverRow<D>>,
        U: Update<'a, Self, D> + Send + Sync,
    >(
        conn: &mut (impl EasyExecutor<D> + Send + Sync),
        update: U,
        where_: impl FnOnce(&mut QueryBuilder<'a, D>) -> anyhow::Result<WhereClause>,
        perform: impl FnMut(T) -> anyhow::Result<Option<Break>> + Send + Sync,
    ) -> anyhow::Result<()>
    where
        DriverRow<D>: ToConvert<D> + 'static,
        for<'b> &'b mut DriverConnection<D>:
            sqlx::Executor<'b, Database = D::InternalDriver> + Send + Sync,
        for<'b> DriverArguments<'b, D>: sqlx::IntoArguments<'b, D::InternalDriver> + Debug,
    {
        let mut builder = QueryBuilder::default();
        let set = update.updates(&mut builder)?;
        let where_ = where_(&mut builder)?;
        let sql = Sql::Update {
            table: Self::table_name(),
            set,
            where_,
        };
        conn.fetch_lazy(sql, builder, perform).await
    }
}
