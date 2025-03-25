use sqlx::sqlite::SqlitePoolOptions;

async fn test() -> anyhow::Result<()> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://test.db")
        .await?;

    let mut conn = pool.begin().await?;

    sqlx::query("SELECT 1").fetch_one(&mut *conn).await?;

    Ok(())
}
