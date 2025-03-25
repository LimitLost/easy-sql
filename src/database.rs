use anyhow::Context;
use easy_macros::{helpers::context, macros::always_context};

use crate::DatabaseSetup;

type Db = sqlx::Sqlite;
type Connection = sqlx::pool::PoolConnection<Db>;

pub struct Database {
    connection_pool: sqlx::Pool<Db>,
}
#[always_context]
impl Database {
    pub async fn setup<T: DatabaseSetup>(url: &str) -> anyhow::Result<Self> {
        let connection_pool = sqlx::Pool::<Db>::connect(url).await?;
        Ok(Database { connection_pool })
    }

    pub async fn conn(&self) -> anyhow::Result<Connection> {
        let conn = self.connection_pool.acquire().await?;
        Ok(conn)
    }

    pub async fn transaction(&self) -> anyhow::Result<sqlx::Transaction<'_, Db>> {
        let conn = self.connection_pool.begin().await?;
        Ok(conn)
    }
}
