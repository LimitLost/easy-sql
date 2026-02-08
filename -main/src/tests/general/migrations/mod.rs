use super::{Database, TestDriver};
use crate::{DatabaseSetup, Insert, Output, Table};
use anyhow::Context;
use easy_macros::always_context;
use easy_sql_macros::query;

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
