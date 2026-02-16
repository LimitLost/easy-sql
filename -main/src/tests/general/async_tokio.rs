use super::{Database, TestDriver};
use crate::{Insert, Output, Table, Update};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::{query, query_lazy};
use futures::StreamExt;

#[derive(Table, Debug, Clone)]
#[sql(no_version)]
struct TokioSpawnTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    group_key: String,
    int_field: i32,
    bool_field: bool,
    nullable_field: Option<String>,
}

#[derive(Insert, Output, Debug, Clone, PartialEq)]
#[sql(table = TokioSpawnTable)]
#[sql(default = id)]
struct TokioSpawnData {
    group_key: String,
    int_field: i32,
    bool_field: bool,
    nullable_field: Option<String>,
}

#[derive(Update, Debug, Clone)]
#[sql(table = TokioSpawnTable)]
struct TokioSpawnUpdate {
    group_key: String,
    int_field: i32,
    bool_field: bool,
    nullable_field: Option<String>,
}

fn spawn_rows() -> Vec<TokioSpawnData> {
    vec![
        TokioSpawnData {
            group_key: "A".to_string(),
            int_field: 10,
            bool_field: true,
            nullable_field: None,
        },
        TokioSpawnData {
            group_key: "A".to_string(),
            int_field: 20,
            bool_field: true,
            nullable_field: Some("x".to_string()),
        },
        TokioSpawnData {
            group_key: "B".to_string(),
            int_field: 30,
            bool_field: false,
            nullable_field: Some("y".to_string()),
        },
    ]
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn mini_test() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    tokio::spawn(async move {
        let mut conn = db.conn().await?;

        let data_vec = spawn_rows();
        query!(conn, INSERT INTO TokioSpawnTable VALUES {data_vec}).await?;

        anyhow::Ok(())
    })
    .await
    .context("tokio::spawn join error")?
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_select_inside_tokio_spawn_specific() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    {
        let mut conn = db.conn().await?;
        let data_vec = spawn_rows();
        query!(conn, INSERT INTO TokioSpawnTable VALUES {data_vec}).await?;
    }

    tokio::spawn(async move {
        let mut conn = db.conn().await?;
        let mut lazy_select = query_lazy!(
            <TestDriver>
            SELECT TokioSpawnData
            FROM TokioSpawnTable
            WHERE int_field >= 20
            ORDER BY int_field DESC
        )?;
        let mut selected = Vec::new();
        {
            let mut stream = lazy_select.fetch(&mut conn);
            while let Some(row) = stream.next().await {
                selected.push(row.context("Failed to fetch selected row")?);
            }
        }

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].int_field, 30);
        assert_eq!(selected[1].int_field, 20);
        anyhow::Ok(())
    })
    .await
    .context("tokio::spawn join error")?
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_update_inside_tokio_spawn_specific() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    {
        let mut conn = db.conn().await?;
        let data = TokioSpawnData {
            group_key: "A".to_string(),
            int_field: 20,
            bool_field: true,
            nullable_field: Some("before".to_string()),
        };
        query!(conn, INSERT INTO TokioSpawnTable VALUES {data}).await?;
    }

    tokio::spawn(async move {
        let mut conn = db.conn().await?;
        let update_data = TokioSpawnUpdate {
            group_key: "A".to_string(),
            int_field: 25,
            bool_field: false,
            nullable_field: Some("after".to_string()),
        };
        let mut lazy_update = query_lazy!(
            <TestDriver>
            UPDATE TokioSpawnTable
            SET {update_data}
            WHERE int_field = 20
            RETURNING TokioSpawnData
        )?;
        let updated = {
            let mut stream = lazy_update.fetch(&mut conn);
            let updated_option = stream.next().await;
            let updated_result = updated_option.context("Expected UPDATE RETURNING row")?;
            updated_result.context("Failed to fetch UPDATE RETURNING row")?
        };

        assert_eq!(updated.int_field, 25);
        assert!(!updated.bool_field);
        assert_eq!(updated.nullable_field.as_deref(), Some("after"));
        anyhow::Ok(())
    })
    .await
    .context("tokio::spawn join error")?
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_delete_inside_tokio_spawn_specific() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    {
        let mut conn = db.conn().await?;
        let data = TokioSpawnData {
            group_key: "A".to_string(),
            int_field: 10,
            bool_field: true,
            nullable_field: Some("delete-me".to_string()),
        };
        query!(conn, INSERT INTO TokioSpawnTable VALUES {data}).await?;
    }

    tokio::spawn(async move {
        let mut conn = db.conn().await?;
        let mut lazy_delete = query_lazy!(
            <TestDriver>
            DELETE FROM TokioSpawnTable
            WHERE int_field = 10
            RETURNING TokioSpawnData
        )?;
        let deleted = {
            let mut stream = lazy_delete.fetch(&mut conn);
            let deleted_option = stream.next().await;
            let deleted_result = deleted_option.context("Expected DELETE RETURNING row")?;
            deleted_result.context("Failed to fetch DELETE RETURNING row")?
        };
        assert_eq!(deleted.int_field, 10);
        let exists_after_delete: bool =
            query!(conn, EXISTS TokioSpawnTable WHERE int_field = 10).await?;
        assert!(!exists_after_delete);
        anyhow::Ok(())
    })
    .await
    .context("tokio::spawn join error")?
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_exists_inside_tokio_spawn_specific() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    {
        let mut conn = db.conn().await?;
        let data = TokioSpawnData {
            group_key: "A".to_string(),
            int_field: 99,
            bool_field: true,
            nullable_field: None,
        };
        query!(conn, INSERT INTO TokioSpawnTable VALUES {data}).await?;
    }

    tokio::spawn(async move {
        let mut conn = db.conn().await?;
        let exists: bool = query!(conn, EXISTS TokioSpawnTable WHERE int_field = 99).await?;
        assert!(exists);

        anyhow::Ok(())
    })
    .await
    .context("tokio::spawn join error")?
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_query_macro_full_coverage_inside_tokio_spawn() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    tokio::spawn(async move {
        let mut conn = db.conn().await?;

        let data_vec = spawn_rows();
        query!(&mut conn, INSERT INTO TokioSpawnTable VALUES {data_vec}).await?;

        let min_int = 10;
        let min_count = 1;
        let limit_rows = 2;
        let selected: Vec<TokioSpawnData> = query!(&mut conn,
            SELECT DISTINCT Vec<TokioSpawnData>
            FROM TokioSpawnTable
            WHERE int_field >= {min_int} AND bool_field = true
            GROUP BY TokioSpawnTable.id
            HAVING COUNT(*) >= {min_count}
            ORDER BY int_field DESC
            LIMIT {limit_rows}
        )
        .await?;
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].int_field, 20);
        assert_eq!(selected[1].int_field, 10);

        let updated_data = TokioSpawnUpdate {
            group_key: "A".to_string(),
            int_field: 25,
            bool_field: true,
            nullable_field: Some("updated".to_string()),
        };
        let updated: TokioSpawnData = query!(&mut conn,
            UPDATE TokioSpawnTable
            SET {updated_data}
            WHERE int_field = 20
            RETURNING TokioSpawnData
        )
        .await?;
        assert_eq!(updated.int_field, 25);
        assert_eq!(updated.nullable_field, Some("updated".to_string()));

        let deleted: TokioSpawnData = query!(&mut conn,
            DELETE FROM TokioSpawnTable
            WHERE int_field = 10
            RETURNING TokioSpawnData
        )
        .await?;
        assert_eq!(deleted.int_field, 10);

        let exists_after_delete: bool =
            query!(&mut conn, EXISTS TokioSpawnTable WHERE int_field = 10).await?;
        assert!(!exists_after_delete);

        let mut tx = db.transaction().await?;
        let tx_data = TokioSpawnData {
            group_key: "TX".to_string(),
            int_field: 77,
            bool_field: true,
            nullable_field: Some("tx".to_string()),
        };
        query!(&mut tx, INSERT INTO TokioSpawnTable VALUES {tx_data}).await?;
        let tx_exists: bool = query!(&mut tx, EXISTS TokioSpawnTable WHERE int_field = 77).await?;
        assert!(tx_exists);
        tx.rollback().await?;

        anyhow::Ok(())
    })
    .await
    .context("tokio::spawn join error")?
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_query_lazy_macro_full_coverage_inside_tokio_spawn() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<TokioSpawnTable>().await?;

    let handle = tokio::spawn(async move {
        let mut conn = db.conn().await?;

        for data in spawn_rows() {
            let mut lazy_insert = query_lazy!(<TestDriver> INSERT INTO TokioSpawnTable VALUES {data} RETURNING TokioSpawnData)?;
            let mut stream = lazy_insert.fetch(&mut conn);
            let inserted_option = stream.next().await;
            let inserted_result = inserted_option.context("Expected INSERT RETURNING row")?;
            let inserted = inserted_result.context("Failed to read INSERT RETURNING row")?;
            assert!(inserted.int_field > 0);
        }

        let threshold = 5;
        let mut lazy_select = query_lazy!(
            <TestDriver>
            SELECT TokioSpawnData FROM TokioSpawnTable
            WHERE int_field > {threshold}
            ORDER BY int_field DESC
            LIMIT 2
        )?;

        let mut selected = Vec::new();
        {
            let mut stream = lazy_select.fetch(&mut conn);
            while let Some(row) = stream.next().await {
                selected.push(row.context("Failed to fetch selected row")?);
            }
        }
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].int_field, 30);
        assert_eq!(selected[1].int_field, 20);

        let new_int = 26;
        let mut lazy_update = query_lazy!(
            <TestDriver>
            UPDATE TokioSpawnTable
            SET int_field = {new_int}
            WHERE int_field = 20
            RETURNING TokioSpawnData
        )?;
        let updated = {
            let mut stream = lazy_update.fetch(&mut conn);
            let updated_option = stream.next().await;
            let updated_result = updated_option.context("Expected UPDATE RETURNING row")?;
            updated_result.context("Failed to fetch UPDATE RETURNING row")?
        };
        assert_eq!(updated.int_field, 26);

        let mut lazy_delete = query_lazy!(
            <TestDriver>
            DELETE FROM TokioSpawnTable
            WHERE int_field = 10
            RETURNING TokioSpawnData
        )?;
        let deleted = {
            let mut stream = lazy_delete.fetch(&mut conn);
            let deleted_option = stream.next().await;
            let deleted_result = deleted_option.context("Expected DELETE RETURNING row")?;
            deleted_result.context("Failed to fetch DELETE RETURNING row")?
        };
        assert_eq!(deleted.int_field, 10);

        let mut tx = db.transaction().await?;
        let tx_data = TokioSpawnData {
            group_key: "TX".to_string(),
            int_field: 88,
            bool_field: true,
            nullable_field: Some("tx-lazy".to_string()),
        };
        let mut tx_lazy_insert = query_lazy!(
            <TestDriver>
            INSERT INTO TokioSpawnTable VALUES {tx_data} RETURNING TokioSpawnData
        )?;
        let tx_inserted = {
            let mut stream = tx_lazy_insert.fetch(&mut tx);
            let row_option = stream.next().await;
            let row_result = row_option.context("Expected tx INSERT RETURNING row")?;
            row_result.context("Failed to fetch tx INSERT RETURNING row")?
        };
        assert_eq!(tx_inserted.int_field, 88);
        tx.rollback().await?;

        anyhow::Ok(())
    });

    let joined = handle.await.context("tokio::spawn join error")?;
    joined.context("spawned query_lazy! workflow failed")?;
    Ok(())
}
