#[cfg(all(feature = "postgres", not(feature = "sqlite")))]
use crate::drivers::postgres::{Database, Postgres as ExampleDriver};

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
use crate::drivers::sqlite::{Database, Sqlite as ExampleDriver};

use crate::{DatabaseSetup, Table};
use easy_macros::{add_code, always_context};

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    Database::setup_for_testing::<DocSchema>().await?;
    assert_eq!(
        <DocUsers as Table<ExampleDriver>>::table_name(),
        "doc_users"
    );
    assert_eq!(
        <DocPosts as Table<ExampleDriver>>::table_name(),
        "doc_posts"
    );
    Ok(())
})]
#[docify::export_content]
#[allow(dead_code)]
async fn database_setup_basic_example() -> anyhow::Result<()> {
    #[derive(Table)]
    struct DocUsers {
        #[sql(primary_key)]
        id: i32,
        email: String,
    }

    #[derive(Table)]
    struct DocPosts {
        #[sql(primary_key)]
        id: i32,
        title: String,
    }

    #[derive(DatabaseSetup)]
    struct DocSchema {
        users: DocUsers,
        posts: DocPosts,
    }
}

#[always_context(skip(!))]
#[no_context]
#[add_code(after = {
    let db = Database::setup_for_testing::<UserTables>().await?;
    let mut conn = db.transaction().await?;
    ContentTables::setup(&mut &mut conn).await?;
    assert_eq!(
        <DocComments as Table<ExampleDriver>>::table_name(),
        "doc_comments"
    );
    Ok(())
})]
#[allow(dead_code)]
#[docify::export_content]
async fn database_setup_nested_example() -> anyhow::Result<()> {
    #[derive(Table)]
    struct DocUsers {
        #[sql(primary_key)]
        id: i32,
        name: String,
    }

    #[derive(Table)]
    struct DocPosts {
        #[sql(primary_key)]
        id: i32,
        #[sql(foreign_key = DocUsers, cascade)]
        user_id: i32,
        title: String,
    }

    #[derive(Table)]
    struct DocComments {
        #[sql(primary_key)]
        id: i32,
        #[sql(foreign_key = DocPosts, cascade)]
        post_id: i32,
        body: String,
    }

    #[derive(DatabaseSetup)]
    #[sql(drivers = ExampleDriver)]
    struct UserTables {
        users: DocUsers,
    }

    #[derive(DatabaseSetup)]
    struct ContentTables {
        posts: DocPosts,
        comments: DocComments,
    }
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_database_setup_basic_example() -> anyhow::Result<()> {
    database_setup_basic_example().await
}

#[always_context(skip(!))]
#[no_context]
#[tokio::test]
async fn test_database_setup_nested_example() -> anyhow::Result<()> {
    database_setup_nested_example().await
}
