use super::{Database, TestDriver};
use crate::{DatabaseSetup, Insert, Output, Table};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;

// Note: this cannot be referenced inside #[sql(unique_id = ...)] because that attribute requires a string literal.
const MIGRATION_REPEAT_VERSIONED_TABLE_ID: &str = "f726b8aa-6d36-4c0a-92c4-9f6f0e8df5d1";

#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2")]
#[sql(table_name = "migration_test_table")]
struct MigrationTestTableV1 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
}

#[derive(Insert)]
#[sql(table = MigrationTestTableV1)]
#[sql(default = id)]
struct MigrationTestInsertV1 {
    name: String,
}

/// Add new column 'age' with default value
#[derive(Table, Debug)]
#[sql(version_test = 2)]
#[sql(unique_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2")]
#[sql(table_name = "migration_test_table")]
struct MigrationTestTableV2 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    #[sql(default = 0)]
    age: i32,
}

#[derive(Insert)]
#[sql(table = MigrationTestTableV2)]
#[sql(default = id)]
struct MigrationTestInsertV2 {
    name: String,
    age: i32,
}

#[derive(Output, Debug)]
#[sql(table = MigrationTestTableV2)]
struct MigrationTestRowV2 {
    id: i32,
    name: String,
    age: i32,
}
/// Add new column 'score' with default value (more than 2 versions test)
#[derive(Table, Debug)]
#[sql(version_test = 3)]
#[sql(unique_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2")]
#[sql(table_name = "migration_test_table")]
struct MigrationTestTableV3 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    #[sql(default = 0)]
    age: i32,
    #[sql(default = 100)]
    score: i32,
}

#[derive(Insert)]
#[sql(table = MigrationTestTableV3)]
#[sql(default = id)]
struct MigrationTestInsertV3 {
    name: String,
    age: i32,
    score: i32,
}

#[derive(Output, Debug)]
#[sql(table = MigrationTestTableV3)]
struct MigrationTestRowV3 {
    id: i32,
    name: String,
    age: i32,
    score: i32,
}
/// Rename column 'name' to 'full_name'
#[derive(Table, Debug)]
#[sql(version_test = 4)]
#[sql(unique_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2")]
#[sql(table_name = "migration_test_table")]
struct MigrationTestTableV4 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    full_name: String,
    #[sql(default = 0)]
    age: i32,
    #[sql(default = 100)]
    score: i32,
}

#[derive(Output, Debug)]
#[sql(table = MigrationTestTableV4)]
struct MigrationTestRowV4 {
    full_name: String,
    age: i32,
    score: i32,
}

#[derive(Insert)]
#[sql(table = MigrationTestTableV4)]
#[sql(default = id)]
struct MigrationTestInsertV4 {
    full_name: String,
    age: i32,
    score: i32,
}
/// Rename table to 'migration_test_table_renamed'
#[derive(Table, Debug)]
#[sql(version_test = 5)]
#[sql(unique_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2")]
#[sql(table_name = "migration_test_table_renamed")]
struct MigrationTestTableV5 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    full_name: String,
    #[sql(default = 0)]
    age: i32,
    #[sql(default = 100)]
    score: i32,
}

#[derive(Output, Debug)]
#[sql(table = MigrationTestTableV5)]
struct MigrationTestRowV5 {
    full_name: String,
    age: i32,
    score: i32,
}

#[derive(Insert)]
#[sql(table = MigrationTestTableV5)]
#[sql(default = id)]
struct MigrationTestInsertV5 {
    full_name: String,
    age: i32,
    score: i32,
}
/// Add new nullable column 'nickname' without default
#[derive(Table, Debug)]
#[sql(version_test = 6)]
#[sql(unique_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2")]
#[sql(table_name = "migration_test_table_renamed")]
struct MigrationTestTableV6 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    full_name: String,
    #[sql(default = 0)]
    age: i32,
    #[sql(default = 100)]
    score: i32,
    nickname: Option<String>,
}

#[derive(Output, Debug)]
#[sql(table = MigrationTestTableV6)]
struct MigrationTestRowV6 {
    full_name: String,
    age: i32,
    score: i32,
    nickname: Option<String>,
}

#[derive(Table, Debug)]
#[sql(no_version)]
#[sql(table_name = "migration_repeat_no_version_table")]
struct MigrationRepeatNoVersionTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
}

#[derive(Insert)]
#[sql(table = MigrationRepeatNoVersionTable)]
#[sql(default = id)]
struct MigrationRepeatNoVersionInsert {
    name: String,
}

#[derive(Output, Debug)]
#[sql(table = MigrationRepeatNoVersionTable)]
struct MigrationRepeatNoVersionRow {
    id: i32,
    name: String,
}

#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "f726b8aa-6d36-4c0a-92c4-9f6f0e8df5d1")]
#[sql(table_name = "migration_repeat_versioned_table")]
struct MigrationRepeatVersionedTableV1 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
}

#[derive(Insert)]
#[sql(table = MigrationRepeatVersionedTableV1)]
#[sql(default = id)]
struct MigrationRepeatVersionedInsertV1 {
    name: String,
}

#[derive(Table, Debug)]
#[sql(version_test = 2)]
#[sql(unique_id = "f726b8aa-6d36-4c0a-92c4-9f6f0e8df5d1")]
#[sql(table_name = "migration_repeat_versioned_table")]
struct MigrationRepeatVersionedTableV2 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    name: String,
    #[sql(default = 0)]
    age: i32,
}

#[derive(Output, Debug)]
#[sql(table = MigrationRepeatVersionedTableV2)]
struct MigrationRepeatVersionedRowV2 {
    id: i32,
    name: String,
    age: i32,
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_add_column_with_default() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV1>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV1 {
        name: "Alice".to_string(),
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV1 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV2> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV2> FROM MigrationTestTableV2 WHERE true ORDER BY id
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(
        rows[0].name, "Alice",
        "Name should be preserved during migration"
    );
    assert_eq!(rows[0].age, 0, "New column should use default value");

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(2),
        "Expected table version to be updated to 2"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_setup_idempotent_no_version_repeat_calls() -> anyhow::Result<()> {
    // Re-running setup for no_version tables should be safe and leave table state intact.
    let db = Database::setup_for_testing::<MigrationRepeatNoVersionTable>().await?;

    let mut conn = db.conn().await?;
    <MigrationRepeatNoVersionTable as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;
    <MigrationRepeatNoVersionTable as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let insert = MigrationRepeatNoVersionInsert {
        name: "repeat-no-version".to_string(),
    };
    query!(&mut conn, INSERT INTO MigrationRepeatNoVersionTable VALUES {insert}).await?;

    let rows: Vec<MigrationRepeatNoVersionRow> = query!(&mut conn,
        SELECT Vec<MigrationRepeatNoVersionRow> FROM MigrationRepeatNoVersionTable WHERE true ORDER BY id
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected exactly one row after repeated setup() calls");
    assert_eq!(rows[0].id, 1, "Expected stable primary key after repeated setup() calls");
    assert_eq!(rows[0].name, "repeat-no-version");

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_setup_idempotent_versioned_v1_to_v2_repeat_calls() -> anyhow::Result<()> {
    // Re-running setup after migration should not reapply the same migration or duplicate data changes.
    let db = Database::setup_for_testing::<MigrationRepeatVersionedTableV1>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationRepeatVersionedInsertV1 {
        name: "repeat-versioned".to_string(),
    };
    query!(&mut tx, INSERT INTO MigrationRepeatVersionedTableV1 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;
    <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationRepeatVersionedRowV2> = query!(&mut conn,
        SELECT Vec<MigrationRepeatVersionedRowV2> FROM MigrationRepeatVersionedTableV2 WHERE true ORDER BY id
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(rows[0].id, 1, "Expected stable primary key after migration + repeated setup() calls");
    assert_eq!(rows[0].name, "repeat-versioned");
    assert_eq!(rows[0].age, 0, "Default value should be applied once");

    let table_id = MIGRATION_REPEAT_VERSIONED_TABLE_ID.to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(version, Some(2), "Expected metadata version to stay at 2 after repeated setup() calls");

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_fresh_setup_for_testing_loop_no_version() -> anyhow::Result<()> {
    for index in 0..2 {
        let db = Database::setup_for_testing::<MigrationRepeatNoVersionTable>().await?;

        let mut conn = db.conn().await?;
        <MigrationRepeatNoVersionTable as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

        let insert = MigrationRepeatNoVersionInsert {
            name: format!("repeat-no-version-{index}"),
        };
        query!(&mut conn, INSERT INTO MigrationRepeatNoVersionTable VALUES {insert}).await?;

        let rows: Vec<MigrationRepeatNoVersionRow> = query!(&mut conn,
            SELECT Vec<MigrationRepeatNoVersionRow> FROM MigrationRepeatNoVersionTable WHERE true ORDER BY id
        )
        .await?;

        assert_eq!(rows.len(), 1, "Each fresh setup_for_testing() database should contain exactly one inserted row");
        assert_eq!(rows[0].id, 1, "Primary key should start from 1 in each fresh setup_for_testing() database");
        assert_eq!(rows[0].name, format!("repeat-no-version-{index}"));
    }

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_fresh_setup_for_testing_loop_versioned_v1_to_v2() -> anyhow::Result<()> {
    for index in 0..2 {
        let db = Database::setup_for_testing::<MigrationRepeatVersionedTableV1>().await?;

        let mut tx = db.transaction().await?;
        let insert = MigrationRepeatVersionedInsertV1 {
            name: format!("repeat-versioned-{index}"),
        };
        query!(&mut tx, INSERT INTO MigrationRepeatVersionedTableV1 VALUES {insert}).await?;
        tx.commit().await?;

        let mut conn = db.conn().await?;
        <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

        let rows: Vec<MigrationRepeatVersionedRowV2> = query!(&mut conn,
            SELECT Vec<MigrationRepeatVersionedRowV2> FROM MigrationRepeatVersionedTableV2 WHERE true ORDER BY id
        )
        .await?;

        assert_eq!(rows.len(), 1, "Each fresh setup_for_testing() database should contain exactly one migrated row");
        assert_eq!(rows[0].id, 1, "Primary key should start from 1 in each fresh setup_for_testing() database");
        assert_eq!(rows[0].name, format!("repeat-versioned-{index}"));
        assert_eq!(rows[0].age, 0, "Migration should apply default age during v1 -> v2");

        let table_id = MIGRATION_REPEAT_VERSIONED_TABLE_ID.to_string();
        let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
        assert_eq!(version, Some(2), "Version metadata should be v2 after migration in each fresh setup_for_testing() loop");
    }

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_setup_concurrent_no_version_is_deterministic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationRepeatNoVersionTable>().await?;

    let left = async {
        let mut conn = db.conn().await?;
        <MigrationRepeatNoVersionTable as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await
    };

    let right = async {
        let mut conn = db.conn().await?;
        <MigrationRepeatNoVersionTable as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await
    };

    let (left_result, right_result) = tokio::join!(left, right);
    left_result.context("left concurrent setup task failed")?;
    right_result.context("right concurrent setup task failed")?;

    let mut conn = db.conn().await?;
    let insert = MigrationRepeatNoVersionInsert {
        name: "concurrent-no-version".to_string(),
    };
    query!(&mut conn, INSERT INTO MigrationRepeatNoVersionTable VALUES {insert}).await?;

    let rows: Vec<MigrationRepeatNoVersionRow> = query!(&mut conn,
        SELECT Vec<MigrationRepeatNoVersionRow> FROM MigrationRepeatNoVersionTable WHERE true ORDER BY id
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected deterministic table state after concurrent setup() calls");
    assert_eq!(rows[0].id, 1, "Expected stable primary key after concurrent setup() calls");

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_setup_concurrent_versioned_is_deterministic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationRepeatVersionedTableV2>().await?;

    let left = async {
        let mut conn = db.conn().await?;
        <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await
    };

    let right = async {
        let mut conn = db.conn().await?;
        <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await
    };

    let (left_result, right_result) = tokio::join!(left, right);
    left_result.context("left concurrent versioned setup task failed")?;
    right_result.context("right concurrent versioned setup task failed")?;

    let mut conn = db.conn().await?;
    let rows: Vec<MigrationRepeatVersionedRowV2> = query!(&mut conn,
        SELECT Vec<MigrationRepeatVersionedRowV2> FROM MigrationRepeatVersionedTableV2 WHERE true ORDER BY id
    )
    .await?;
    assert!(
        rows.is_empty(),
        "Expected deterministic empty data set after concurrent setup() calls on fresh table"
    );

    let table_id = MIGRATION_REPEAT_VERSIONED_TABLE_ID.to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(2),
        "Expected metadata version to remain at target after concurrent setup() calls"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_setup_concurrent_first_time_versioned_is_deterministic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<crate::EasySqlTables>().await?;

    let left = async {
        let mut conn = db.conn().await?;
        <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await
    };

    let right = async {
        let mut conn = db.conn().await?;
        <MigrationRepeatVersionedTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await
    };

    let (left_result, right_result) = tokio::join!(left, right);
    left_result.context("left concurrent first-time versioned setup task failed")?;
    right_result.context("right concurrent first-time versioned setup task failed")?;

    let mut conn = db.conn().await?;
    let rows: Vec<MigrationRepeatVersionedRowV2> = query!(&mut conn,
        SELECT Vec<MigrationRepeatVersionedRowV2> FROM MigrationRepeatVersionedTableV2 WHERE true ORDER BY id
    )
    .await?;
    assert!(
        rows.is_empty(),
        "Expected deterministic empty data set after first-time concurrent setup() calls"
    );

    let table_id = MIGRATION_REPEAT_VERSIONED_TABLE_ID.to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(2),
        "Expected metadata version to be at target after first-time concurrent setup() calls"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_v1_to_v3_defaults_applied() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV1>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV1 {
        name: "Alice".to_string(),
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV1 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV3 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV3> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV3> FROM MigrationTestTableV3 WHERE true ORDER BY id
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(rows[0].name, "Alice", "Name should be preserved");
    assert_eq!(rows[0].age, 0, "Age should use the default value");
    assert_eq!(rows[0].score, 100, "Score should use the default value");

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();

    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(3),
        "Expected table version to be updated to 3"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_v2_to_v3_preserves_existing_data() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV2>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV2 {
        name: "Bob".to_string(),
        age: 42,
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV2 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV3 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV3> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV3> FROM MigrationTestTableV3 WHERE name = "Bob"
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(rows[0].age, 42, "Existing age value should be preserved");
    assert_eq!(rows[0].score, 100, "New column should use default value");

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(3),
        "Expected table version to be updated to 3"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_rename_column_preserves_data() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV3>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV3 {
        name: "Carol".to_string(),
        age: 30,
        score: 250,
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV3 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV4 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV4> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV4> FROM MigrationTestTableV4 WHERE full_name = "Carol"
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(
        rows[0].full_name, "Carol",
        "Renamed column should keep value"
    );
    assert_eq!(rows[0].age, 30, "Existing age value should be preserved");
    assert_eq!(
        rows[0].score, 250,
        "Existing score value should be preserved"
    );

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(4),
        "Expected table version to be updated to 4"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_rename_table_preserves_data() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV4>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV4 {
        full_name: "Diana".to_string(),
        age: 28,
        score: 180,
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV4 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV5 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV5> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV5> FROM MigrationTestTableV5 WHERE full_name = "Diana"
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(
        rows[0].full_name, "Diana",
        "Data should persist after table rename"
    );
    assert_eq!(rows[0].age, 28, "Existing age value should be preserved");
    assert_eq!(
        rows[0].score, 180,
        "Existing score value should be preserved"
    );

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(5),
        "Expected table version to be updated to 5"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_add_nullable_column_without_default() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV5>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV5 {
        full_name: "Eve".to_string(),
        age: 21,
        score: 75,
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV5 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV6 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV6> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV6> FROM MigrationTestTableV6 WHERE full_name = "Eve"
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(
        rows[0].full_name, "Eve",
        "Existing name value should be preserved"
    );
    assert_eq!(rows[0].age, 21, "Existing age value should be preserved");
    assert_eq!(
        rows[0].score, 75,
        "Existing score value should be preserved"
    );
    assert!(
        rows[0].nickname.is_none(),
        "New nullable column should be NULL by default"
    );

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(6),
        "Expected table version to be updated to 6"
    );

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_v1_to_v3_to_v6_with_update() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<MigrationTestTableV1>().await?;

    let mut tx = db.transaction().await?;
    let insert = MigrationTestInsertV1 {
        name: "Alice".to_string(),
    };
    query!(&mut tx, INSERT INTO MigrationTestTableV1 VALUES {insert}).await?;
    tx.commit().await?;

    let mut conn = db.conn().await?;
    <MigrationTestTableV3 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    query!(&mut conn,
        UPDATE MigrationTestTableV3 SET age = 27, score = 150 WHERE name = "Alice"
    )
    .await?;

    <MigrationTestTableV6 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let rows: Vec<MigrationTestRowV6> = query!(&mut conn,
        SELECT Vec<MigrationTestRowV6> FROM MigrationTestTableV6 WHERE full_name = "Alice"
    )
    .await?;

    assert_eq!(rows.len(), 1, "Expected a single migrated row");
    assert_eq!(rows[0].age, 27, "Updated age should be preserved");
    assert_eq!(rows[0].score, 150, "Updated score should be preserved");
    assert!(
        rows[0].nickname.is_none(),
        "Nickname should remain NULL after migration"
    );

    let table_id = "9e0ab3c7-2e5d-4f13-b6d8-7c8ea17a3cf2".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(
        version,
        Some(6),
        "Expected table version to be updated to 6"
    );

    Ok(())
}

// ---- Foreign-key migration: adding a foreign key to an existing table (SQLite table rebuild) ----

#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "a1b2c3d4-0001-4abc-8def-000000000001")]
#[sql(table_name = "fk_rebuild_parent")]
struct FkRebuildParent {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    label: String,
}

#[derive(Insert)]
#[sql(table = FkRebuildParent)]
#[sql(default = id)]
struct FkRebuildParentInsert {
    label: String,
}

/// Child table v1 — no foreign key yet.
#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "a1b2c3d4-0002-4abc-8def-000000000002")]
#[sql(table_name = "fk_rebuild_child")]
struct FkRebuildChildV1 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    parent_id: i32,
    note: String,
}

#[derive(Insert)]
#[sql(table = FkRebuildChildV1)]
#[sql(default = id)]
struct FkRebuildChildInsert {
    parent_id: i32,
    note: String,
}

/// Child table v2 — adds a cascading foreign key on `parent_id`, exercising the table-rebuild migration.
#[derive(Table, Debug)]
#[sql(version_test = 2)]
#[sql(unique_id = "a1b2c3d4-0002-4abc-8def-000000000002")]
#[sql(table_name = "fk_rebuild_child")]
struct FkRebuildChildV2 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(foreign_key = FkRebuildParent, cascade)]
    parent_id: i32,
    note: String,
}

#[derive(Output, Debug)]
#[sql(table = FkRebuildChildV2)]
struct FkRebuildChildRow {
    id: i32,
    parent_id: i32,
    note: String,
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_add_foreign_key_preserves_data() -> anyhow::Result<()> {
    // Step 1: bring up the parent + child(v1) and seed a valid (non-orphan) child row.
    let db = Database::setup_for_testing::<FkRebuildParent>().await?;
    let mut conn = db.conn().await?;
    <FkRebuildChildV1 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let parent = FkRebuildParentInsert {
        label: "p1".to_string(),
    };
    query!(&mut conn, INSERT INTO FkRebuildParent VALUES {parent}).await?;
    let child = FkRebuildChildInsert {
        parent_id: 1,
        note: "kept".to_string(),
    };
    query!(&mut conn, INSERT INTO FkRebuildChildV1 VALUES {child}).await?;

    // Step 2: migrate child v1 -> v2 (adds the foreign key via a full table rebuild).
    <FkRebuildChildV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    // Step 3: the rebuild preserved the row verbatim.
    let rows: Vec<FkRebuildChildRow> = query!(&mut conn,
        SELECT Vec<FkRebuildChildRow> FROM FkRebuildChildV2 WHERE true ORDER BY id
    )
    .await?;
    assert_eq!(rows.len(), 1, "the rebuilt table keeps its row");
    assert_eq!(rows[0].parent_id, 1);
    assert_eq!(rows[0].note, "kept", "column data survives the rebuild");

    // Step 4: the version advanced to 2.
    let table_id = "a1b2c3d4-0002-4abc-8def-000000000002".to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(version, Some(2), "child table version is bumped to 2");

    Ok(())
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_add_foreign_key_fails_loudly_on_orphans() -> anyhow::Result<()> {
    // Step 1: parent table exists but is empty; seed a child that references a non-existent parent (an orphan).
    let db = Database::setup_for_testing::<FkRebuildParent>().await?;
    let mut conn = db.conn().await?;
    <FkRebuildChildV1 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    let orphan = FkRebuildChildInsert {
        parent_id: 999,
        note: "orphan".to_string(),
    };
    query!(&mut conn, INSERT INTO FkRebuildChildV1 VALUES {orphan}).await?;

    // Step 2: migrating to v2 must FAIL — `foreign_key_check` sees the orphan row violating the new constraint.
    // (Proves the rebuilt table really carries the foreign key, and that orphans are surfaced, not swallowed.)
    let result = <FkRebuildChildV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await;
    assert!(
        result.is_err(),
        "adding a foreign key over orphaned rows must fail the migration loudly"
    );

    Ok(())
}

/// Self-referential table v1 — no foreign key yet.
#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "a1b2c3d4-0003-4abc-8def-000000000003")]
#[sql(table_name = "fk_selfref")]
struct FkSelfRefV1 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    parent_id: Option<i32>,
    name: String,
}

#[derive(Insert)]
#[sql(table = FkSelfRefV1)]
#[sql(default = id)]
struct FkSelfRefInsert {
    parent_id: Option<i32>,
    name: String,
}

/// Self-referential table v2 — adds a foreign key on `parent_id` that references this same table (the trickiest
/// rebuild case: the injected `REFERENCES fk_selfref` resolves to the rebuilt table itself after the swap).
#[derive(Table, Debug)]
#[sql(version_test = 2)]
#[sql(unique_id = "a1b2c3d4-0003-4abc-8def-000000000003")]
#[sql(table_name = "fk_selfref")]
struct FkSelfRefV2 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(foreign_key = FkSelfRefV2, cascade)]
    parent_id: Option<i32>,
    name: String,
}

#[derive(Output, Debug)]
#[sql(table = FkSelfRefV2)]
struct FkSelfRefRow {
    id: i32,
    parent_id: Option<i32>,
    name: String,
}

#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_add_self_referential_foreign_key() -> anyhow::Result<()> {
    // Step 1: build a small hierarchy in v1 (a root + a child pointing at it).
    let db = Database::setup_for_testing::<FkSelfRefV1>().await?;
    let mut conn = db.conn().await?;

    let root = FkSelfRefInsert {
        parent_id: None,
        name: "root".to_string(),
    };
    query!(&mut conn, INSERT INTO FkSelfRefV1 VALUES {root}).await?;
    let child = FkSelfRefInsert {
        parent_id: Some(1),
        name: "child".to_string(),
    };
    query!(&mut conn, INSERT INTO FkSelfRefV1 VALUES {child}).await?;

    // Step 2: migrate to v2 — adds the self-referential foreign key via the table rebuild.
    <FkSelfRefV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn).await?;

    // Step 3: both rows survive, and the parent/child link is intact.
    let rows: Vec<FkSelfRefRow> = query!(&mut conn,
        SELECT Vec<FkSelfRefRow> FROM FkSelfRefV2 WHERE true ORDER BY id
    )
    .await?;
    assert_eq!(rows.len(), 2, "the self-referential rebuild keeps both rows");
    assert_eq!(rows[0].parent_id, None, "root keeps a NULL parent");
    assert_eq!(rows[1].parent_id, Some(1), "child still points at the root");

    Ok(())
}

// ==============================================
// #[sql(bytes)] storage-compatibility migrations
// ==============================================
// A `#[sql(bytes)]` field may change its Rust wrapper type across versions while still persisting the same
// (nullable) blob column. Migration generation must treat that as a storage-compatible no-op instead of the
// unsupported "type change" error. These version_test tables would fail to BUILD if the guard rejected the
// change, so a successful compile is itself part of the proof; the tests add the runtime half.

const BYTES_COMPAT_TABLE_ID: &str = "b17e5c0a-0001-4d2e-9a3f-000000000001";

#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "b17e5c0a-0001-4d2e-9a3f-000000000001")]
#[sql(table_name = "bytes_compat_table")]
struct BytesCompatTableV1 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(bytes)]
    payload: Vec<String>,
}

#[derive(Insert)]
#[sql(table = BytesCompatTableV1)]
#[sql(default = id)]
struct BytesCompatInsertV1 {
    #[sql(bytes)]
    payload: Vec<String>,
}

// V2: same table, non-null bytes payload wrapper changed `Vec<String>` -> `HashMap<String, i32>`. Both persist
// as a non-null blob, so generation must consider it storage-compatible (no ALTER, version bump only).
#[derive(Table, Debug)]
#[sql(version_test = 2)]
#[sql(unique_id = "b17e5c0a-0001-4d2e-9a3f-000000000001")]
#[sql(table_name = "bytes_compat_table")]
struct BytesCompatTableV2 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(bytes)]
    payload: std::collections::HashMap<String, i32>,
}

// Reads only the id: a v1 row holds a `Vec<String>`-serialized blob, so decoding it as the v2 type is
// intentionally out of scope (bytes-> a different wrapper is user-managed re-encoding).
#[derive(Output, Debug)]
#[sql(table = BytesCompatTableV2)]
struct BytesCompatIdRow {
    id: i32,
}

const BYTES_COMPAT_OPT_TABLE_ID: &str = "b17e5c0a-0002-4d2e-9a3f-000000000002";

#[derive(Table, Debug)]
#[sql(version_test = 1)]
#[sql(unique_id = "b17e5c0a-0002-4d2e-9a3f-000000000002")]
#[sql(table_name = "bytes_compat_opt_table")]
struct BytesCompatOptTableV1 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(bytes)]
    payload: Option<Vec<String>>,
}

#[derive(Insert)]
#[sql(table = BytesCompatOptTableV1)]
#[sql(default = id)]
struct BytesCompatOptInsertV1 {
    #[sql(bytes)]
    payload: Option<Vec<String>>,
}

// V2: optional bytes payload wrapper changed `Option<Vec<String>>` -> `Option<HashMap<String, i32>>`. Both
// persist as a NULLABLE blob; normalization preserves the optionality, so they stay storage-compatible.
#[derive(Table, Debug)]
#[sql(version_test = 2)]
#[sql(unique_id = "b17e5c0a-0002-4d2e-9a3f-000000000002")]
#[sql(table_name = "bytes_compat_opt_table")]
struct BytesCompatOptTableV2 {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    id: i32,
    #[sql(bytes)]
    payload: Option<std::collections::HashMap<String, i32>>,
}

#[derive(Output, Debug)]
#[sql(table = BytesCompatOptTableV2)]
struct BytesCompatOptIdRow {
    id: i32,
}

/// A non-null `#[sql(bytes)]` field changing its wrapper type across versions migrates as a no-op: the blob
/// column is untouched, the existing row survives, and the version advances to 2.
#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_bytes_wrapper_change_is_storage_compatible() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesCompatTableV1>().await?;
    let mut conn = db.conn().await?;

    let insert = BytesCompatInsertV1 {
        payload: vec!["a".to_string(), "b".to_string()],
    };
    query!(&mut conn, INSERT INTO BytesCompatTableV1 VALUES {insert}).await?;

    // Migrate v1 -> v2 (bytes wrapper Vec<String> -> HashMap<String, i32>): storage-compatible, must succeed.
    <BytesCompatTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn)
        .await
        .context("bytes wrapper change should migrate as a storage-compatible no-op")?;

    let rows: Vec<BytesCompatIdRow> = query!(&mut conn,
        SELECT Vec<BytesCompatIdRow> FROM BytesCompatTableV2 WHERE true ORDER BY id
    )
    .await?;
    assert_eq!(rows.len(), 1, "the row survives the storage-compatible migration");
    assert_eq!(rows[0].id, 1);

    let table_id = BYTES_COMPAT_TABLE_ID.to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(version, Some(2), "version advances to 2");

    Ok(())
}

/// Same for an OPTIONAL `#[sql(bytes)]` field: `Option<A>` -> `Option<B>` stays storage-compatible (nullable
/// blob, optionality preserved), so it also migrates as a no-op.
#[always_context(skip(!))]
#[tokio::test]
async fn test_migration_optional_bytes_wrapper_change_is_storage_compatible() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<BytesCompatOptTableV1>().await?;
    let mut conn = db.conn().await?;

    let insert = BytesCompatOptInsertV1 {
        payload: Some(vec!["x".to_string()]),
    };
    query!(&mut conn, INSERT INTO BytesCompatOptTableV1 VALUES {insert}).await?;

    <BytesCompatOptTableV2 as DatabaseSetup<TestDriver>>::setup(&mut &mut conn)
        .await
        .context("optional bytes wrapper change should migrate as a storage-compatible no-op")?;

    let rows: Vec<BytesCompatOptIdRow> = query!(&mut conn,
        SELECT Vec<BytesCompatOptIdRow> FROM BytesCompatOptTableV2 WHERE true ORDER BY id
    )
    .await?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, 1);

    let table_id = BYTES_COMPAT_OPT_TABLE_ID.to_string();
    let version = crate::EasySqlTables_get_version!(TestDriver, &mut conn, table_id);
    assert_eq!(version, Some(2));

    Ok(())
}
