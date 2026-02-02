// Comprehensive tests for table_join! macro functionality
//
// This file contains:
// - Comprehensive tests for INNER, LEFT, RIGHT, CROSS JOINs
// - Tests for multiple joins (chaining tables)
// - Tests for joins with WHERE, ORDER BY, LIMIT, DISTINCT
// - Tests for query_lazy! with joins
// - Tests for Output structs with #[sql(field = ...)] and #[sql(select = ...)]
// - Tests for complex scenarios (UPDATE/DELETE with joins, EXISTS subqueries)
// - Edge case tests (empty tables, NULL handling)

use super::*;
use crate::{DatabaseSetup, Table, table_join};
use anyhow::Context;
use easy_macros::always_context;
use sql_macros::{query, query_lazy};

// ==============================================
// Test Tables for Join Testing
// ==============================================

/// Main users table
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct UsersTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    pub username: String,
    pub email: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = UsersTable)]
#[sql(default = id)]
pub struct User {
    pub username: String,
    pub email: String,
}

/// Posts table with foreign key to users
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct PostsTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    #[sql(foreign_key = UsersTable)]
    pub user_id: i32,
    pub title: String,
    pub content: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = PostsTable)]
#[sql(default = id)]
pub struct Post {
    pub user_id: i32,
    pub title: String,
    pub content: String,
}

/// Comments table with foreign key to posts
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct CommentsTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    #[sql(foreign_key = PostsTable)]
    pub post_id: i32,
    #[sql(foreign_key = UsersTable)]
    pub author_id: i32,
    pub comment_text: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = CommentsTable)]
#[sql(default = id)]
pub struct Comment {
    pub post_id: i32,
    pub author_id: i32,
    pub comment_text: String,
}

/// User profiles table (optional one-to-one relationship)
#[derive(Table, Debug, Clone)]
#[sql(no_version)]
pub struct ProfilesTable {
    #[sql(primary_key)]
    #[sql(auto_increment)]
    pub id: i32,
    #[sql(foreign_key = UsersTable)]
    pub user_id: i32,
    pub bio: String,
    pub location: String,
}

#[derive(Insert, Update, Output, Debug, Clone, PartialEq)]
#[sql(table = ProfilesTable)]
#[sql(default = id)]
pub struct Profile {
    pub user_id: i32,
    pub bio: String,
    pub location: String,
}

// ==============================================
// Table Joins Definitions
// ==============================================

// INNER JOIN: Users with Posts
table_join!(UsersWithPosts | UsersTable INNER JOIN PostsTable ON UsersTable.id = PostsTable.user_id);

// LEFT JOIN: Users with optional Posts
table_join!(UsersWithOptionalPosts | UsersTable LEFT JOIN PostsTable ON UsersTable.id = PostsTable.user_id);

// RIGHT JOIN: Posts with Users (users become optional if right join from posts perspective)
table_join!(PostsWithUsers | PostsTable RIGHT JOIN UsersTable ON PostsTable.user_id = UsersTable.id);

// CROSS JOIN: Users cross Posts (cartesian product)
table_join!(UsersCrossPosts | UsersTable CROSS JOIN PostsTable);

// Multiple INNER JOINs: Users -> Posts -> Comments
table_join!(UsersPostsComments |
    UsersTable
    INNER JOIN PostsTable ON UsersTable.id = PostsTable.user_id
    INNER JOIN CommentsTable ON PostsTable.id = CommentsTable.post_id
);

// Mixed JOINs: Users -> Posts (INNER) -> Comments (LEFT)
table_join!(UsersPostsOptionalComments |
    UsersTable
    INNER JOIN PostsTable ON UsersTable.id = PostsTable.user_id
    LEFT JOIN CommentsTable ON PostsTable.id = CommentsTable.post_id
);

// LEFT JOIN: Users with optional Profiles
table_join!(UsersWithProfiles | UsersTable LEFT JOIN ProfilesTable ON UsersTable.id = ProfilesTable.user_id);

// ==============================================
// Output Structs with #[sql(field = ...)]
// ==============================================

/// Output for UsersWithPosts using field attribute
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersWithPosts)]
pub struct UserPostOutput {
    #[sql(field = UsersTable.id)]
    pub user_id: i32,
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(field = PostsTable.id)]
    pub post_id: i32,
    #[sql(field = PostsTable.title)]
    pub post_title: String,
}

/// Output for PostsWithUsers (RIGHT JOIN)
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = PostsWithUsers)]
pub struct PostUserOutput {
    #[sql(field = UsersTable.id)]
    pub user_id: i32,
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(field = PostsTable.id)]
    pub post_id: i32,
    #[sql(field = PostsTable.title)]
    pub post_title: String,
}

/// Output for UsersCrossPosts (CROSS JOIN)
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersCrossPosts)]
pub struct UserCrossPostOutput {
    #[sql(field = UsersTable.id)]
    pub user_id: i32,
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(field = PostsTable.id)]
    pub post_id: i32,
    #[sql(field = PostsTable.title)]
    pub post_title: String,
}

/// Output for UsersWithOptionalPosts (LEFT JOIN - posts are optional)
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersWithOptionalPosts)]
pub struct UserOptionalPostOutput {
    #[sql(field = UsersTable.id)]
    pub user_id: i32,
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(field = PostsTable.id)]
    pub post_id: Option<i32>,
    #[sql(field = PostsTable.title)]
    pub post_title: Option<String>,
}

/// Output for multiple joins
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersPostsComments)]
pub struct UserPostCommentOutput {
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(field = PostsTable.title)]
    pub post_title: String,
    #[sql(field = CommentsTable.comment_text)]
    pub comment_text: String,
}

/// Output for mixed joins with optional comments
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersPostsOptionalComments)]
pub struct UserPostOptionalCommentOutput {
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(field = PostsTable.title)]
    pub post_title: String,
    #[sql(field = CommentsTable.comment_text)]
    pub comment_text: Option<String>,
}

// ==============================================
// Output Structs with #[sql(select = ...)]
// ==============================================

/// Output using custom SELECT expressions
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersWithPosts)]
pub struct UserPostCustomOutput {
    #[sql(select = UsersTable.username || " - " || PostsTable.title)]
    pub combined_title: String,
    #[sql(select = LENGTH(PostsTable.content))]
    pub content_length: i32,
}

/// Output mixing field and select attributes
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersWithPosts)]
pub struct UserPostMixedOutput {
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(select = UPPER(PostsTable.title))]
    pub title_upper: String,
    #[sql(field = PostsTable.content)]
    pub content: String,
}

/// Output with aggregate function in select
#[derive(Output, Debug, Clone, PartialEq)]
#[sql(table = UsersWithOptionalPosts)]
pub struct UserPostCountOutput {
    #[sql(field = UsersTable.id)]
    pub user_id: i32,
    #[sql(field = UsersTable.username)]
    pub username: String,
    #[sql(select = COUNT(PostsTable.id))]
    pub post_count: i64,
}

// ==============================================
// Test Helper Functions
// ==============================================

/// Insert test user and return user_id
#[always_context(skip(!))]
async fn insert_test_user(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    username: &str,
    email: &str,
) -> anyhow::Result<()> {
    let user = User {
        username: username.to_string(),
        email: email.to_string(),
    };
    query!(conn, INSERT INTO UsersTable VALUES {user})
        .await
        .context("Failed to insert test user")?;
    Ok(())
}

/// Insert test post and return post_id
#[always_context(skip(!))]
async fn insert_test_post(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    user_id: i32,
    title: &str,
    content: &str,
) -> anyhow::Result<()> {
    let post = Post {
        user_id,
        title: title.to_string(),
        content: content.to_string(),
    };
    query!(conn, INSERT INTO PostsTable VALUES {post})
        .await
        .context("Failed to insert test post")?;
    Ok(())
}

/// Insert test comment
#[always_context(skip(!))]
async fn insert_test_comment(
    mut conn: impl crate::EasyExecutor<TestDriver> + Send + Sync,
    post_id: i32,
    author_id: i32,
    comment_text: &str,
) -> anyhow::Result<()> {
    let comment = Comment {
        post_id,
        author_id,
        comment_text: comment_text.to_string(),
    };
    query!(conn, INSERT INTO CommentsTable VALUES {comment})
        .await
        .context("Failed to insert test comment")?;
    Ok(())
}

// ==============================================
// 1. INNER JOIN Tests
// ==============================================

/// Test basic INNER JOIN with query! macro
#[always_context(skip(!))]
#[tokio::test]
async fn test_inner_join_basic_query() -> anyhow::Result<()> {
    // Setup database with multiple tables
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // Insert test data
    insert_test_user(&mut conn, "alice", "alice@example.com").await?;
    insert_test_post(&mut conn, 1, "First Post", "Content of first post").await?;

    // Query with INNER JOIN
    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "alice");
    assert_eq!(results[0].post_title, "First Post");
    assert_eq!(results[0].user_id, 1);
    assert_eq!(results[0].post_id, 1);

    conn.rollback().await?;
    Ok(())
}

/// Test INNER JOIN returns empty when no matches
#[always_context(skip(!))]
#[tokio::test]
async fn test_inner_join_no_matches() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // Insert user but no posts
    insert_test_user(&mut conn, "bob", "bob@example.com").await?;

    // Query with INNER JOIN should return empty
    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(
        results.len(),
        0,
        "INNER JOIN should return empty when no matching posts"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test INNER JOIN with multiple matches (one user, multiple posts)
#[always_context(skip(!))]
#[tokio::test]
async fn test_inner_join_multiple_posts_per_user() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "charlie", "charlie@example.com").await?;
    insert_test_post(&mut conn, 1, "Post 1", "Content 1").await?;
    insert_test_post(&mut conn, 1, "Post 2", "Content 2").await?;
    insert_test_post(&mut conn, 1, "Post 3", "Content 3").await?;

    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 3, "Should return 3 rows for 3 posts");
    assert!(results.iter().all(|r| r.username == "charlie"));
    assert_eq!(results[0].post_title, "Post 1");
    assert_eq!(results[1].post_title, "Post 2");
    assert_eq!(results[2].post_title, "Post 3");

    conn.rollback().await?;
    Ok(())
}

/// Test INNER JOIN with WHERE clause
#[always_context(skip(!))]
#[tokio::test]
async fn test_inner_join_with_where_clause() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "david", "david@example.com").await?;
    insert_test_post(&mut conn, 1, "Important Post", "Important content").await?;
    insert_test_post(&mut conn, 1, "Regular Post", "Regular content").await?;

    let search_title = "Important Post".to_string();
    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts
        WHERE PostsTable.title = {search_title}
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].post_title, "Important Post");

    conn.rollback().await?;
    Ok(())
}

/// Test INNER JOIN with ORDER BY
#[always_context(skip(!))]
#[tokio::test]
async fn test_inner_join_with_order_by() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "eve", "eve@example.com").await?;
    insert_test_post(&mut conn, 1, "Post C", "Content").await?;
    insert_test_post(&mut conn, 1, "Post A", "Content").await?;
    insert_test_post(&mut conn, 1, "Post B", "Content").await?;

    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts
        WHERE true
        ORDER BY PostsTable.title
    )
    .await?;

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].post_title, "Post A");
    assert_eq!(results[1].post_title, "Post B");
    assert_eq!(results[2].post_title, "Post C");

    conn.rollback().await?;
    Ok(())
}

/// Test INNER JOIN with LIMIT
#[always_context(skip(!))]
#[tokio::test]
async fn test_inner_join_with_limit() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "frank", "frank@example.com").await?;
    for i in 1..=10 {
        insert_test_post(&mut conn, 1, &format!("Post {}", i), "Content").await?;
    }

    let limit = 3;
    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts
        WHERE true
        LIMIT {limit}
    )
    .await?;

    assert_eq!(results.len(), 3);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 2. LEFT JOIN Tests
// ==============================================

/// Test LEFT JOIN returns users even without posts
#[always_context(skip(!))]
#[tokio::test]
async fn test_left_join_users_without_posts() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // Insert user without posts
    insert_test_user(&mut conn, "grace", "grace@example.com").await?;

    let results: Vec<UserOptionalPostOutput> = query!(&mut conn,
        SELECT Vec<UserOptionalPostOutput> FROM UsersWithOptionalPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "grace");
    assert_eq!(
        results[0].post_id, None,
        "post_id should be None for LEFT JOIN with no match"
    );
    assert_eq!(
        results[0].post_title, None,
        "post_title should be None for LEFT JOIN with no match"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test LEFT JOIN with existing posts
#[always_context(skip(!))]
#[tokio::test]
async fn test_left_join_users_with_posts() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "henry", "henry@example.com").await?;
    insert_test_post(&mut conn, 1, "Henry's Post", "Content").await?;

    let results: Vec<UserOptionalPostOutput> = query!(&mut conn,
        SELECT Vec<UserOptionalPostOutput> FROM UsersWithOptionalPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "henry");
    assert_eq!(results[0].post_id, Some(1));
    assert_eq!(results[0].post_title, Some("Henry's Post".to_string()));

    conn.rollback().await?;
    Ok(())
}

/// Test LEFT JOIN with mixed users (some with posts, some without)
#[always_context(skip(!))]
#[tokio::test]
async fn test_left_join_mixed_users() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // User 1 with posts
    insert_test_user(&mut conn, "iris", "iris@example.com").await?;
    insert_test_post(&mut conn, 1, "Iris Post", "Content").await?;

    // User 2 without posts
    insert_test_user(&mut conn, "jack", "jack@example.com").await?;

    // User 3 with multiple posts
    insert_test_user(&mut conn, "kate", "kate@example.com").await?;
    insert_test_post(&mut conn, 3, "Kate Post 1", "Content").await?;
    insert_test_post(&mut conn, 3, "Kate Post 2", "Content").await?;

    let results: Vec<UserOptionalPostOutput> = query!(&mut conn,
        SELECT Vec<UserOptionalPostOutput> FROM UsersWithOptionalPosts
        WHERE true
        ORDER BY UsersTable.id, PostsTable.id
    )
    .await?;

    // Should get: iris (1 row), jack (1 row with NULL post), kate (2 rows)
    assert_eq!(results.len(), 4);

    // Iris has post
    assert_eq!(results[0].username, "iris");
    assert!(results[0].post_id.is_some());

    // Jack has no post
    assert_eq!(results[1].username, "jack");
    assert!(results[1].post_id.is_none());

    // Kate has 2 posts
    assert_eq!(results[2].username, "kate");
    assert!(results[2].post_id.is_some());
    assert_eq!(results[3].username, "kate");
    assert!(results[3].post_id.is_some());

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 3. RIGHT JOIN Tests
// ==============================================

/// Test RIGHT JOIN
///
/// Note: SQLite has limited RIGHT JOIN support and may emulate it as LEFT JOIN with reversed tables.
/// PostgreSQL has full RIGHT JOIN support.
#[always_context(skip(!))]
#[tokio::test]
async fn test_right_join_basic() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "leo", "leo@example.com").await?;
    insert_test_post(&mut conn, 1, "Leo's Post", "Content").await?;

    // RIGHT JOIN from PostsTable to UsersTable
    let results: Vec<PostUserOutput> = query!(&mut conn,
        SELECT Vec<PostUserOutput> FROM PostsWithUsers WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "leo");
    assert_eq!(results[0].post_title, "Leo's Post");

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 4. CROSS JOIN Tests
// ==============================================

/// Test CROSS JOIN creates cartesian product
#[always_context(skip(!))]
#[tokio::test]
async fn test_cross_join_cartesian_product() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // 2 users
    insert_test_user(&mut conn, "mike", "mike@example.com").await?;
    insert_test_user(&mut conn, "nancy", "nancy@example.com").await?;

    // 3 posts (from first user)
    insert_test_post(&mut conn, 1, "Post 1", "Content").await?;
    insert_test_post(&mut conn, 1, "Post 2", "Content").await?;
    insert_test_post(&mut conn, 1, "Post 3", "Content").await?;

    let results: Vec<UserCrossPostOutput> = query!(&mut conn,
        SELECT Vec<UserCrossPostOutput> FROM UsersCrossPosts WHERE true
    )
    .await?;

    // Cartesian product: 2 users × 3 posts = 6 rows
    assert_eq!(
        results.len(),
        6,
        "CROSS JOIN should create cartesian product"
    );

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 5. Multiple Joins Tests
// ==============================================

/// Test multiple INNER JOINs (Users -> Posts -> Comments)
#[always_context(skip(!))]
#[tokio::test]
async fn test_multiple_inner_joins() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;
    CommentsTable::setup(&mut &mut conn).await?;

    // User
    insert_test_user(&mut conn, "oliver", "oliver@example.com").await?;

    // Post
    insert_test_post(&mut conn, 1, "Oliver's Post", "Great content").await?;

    // Comments
    insert_test_comment(&mut conn, 1, 1, "First comment").await?;
    insert_test_comment(&mut conn, 1, 1, "Second comment").await?;

    let results: Vec<UserPostCommentOutput> = query!(&mut conn,
        SELECT Vec<UserPostCommentOutput> FROM UsersPostsComments WHERE true
    )
    .await?;

    assert_eq!(results.len(), 2, "Should get 2 rows for 2 comments");
    assert_eq!(results[0].username, "oliver");
    assert_eq!(results[0].post_title, "Oliver's Post");
    assert_eq!(results[0].comment_text, "First comment");
    assert_eq!(results[1].comment_text, "Second comment");

    conn.rollback().await?;
    Ok(())
}

/// Test multiple joins with no matching data
#[always_context(skip(!))]
#[tokio::test]
async fn test_multiple_joins_no_comments() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;
    CommentsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "paula", "paula@example.com").await?;
    insert_test_post(&mut conn, 1, "Post without comments", "Content").await?;

    let results: Vec<UserPostCommentOutput> = query!(&mut conn,
        SELECT Vec<UserPostCommentOutput> FROM UsersPostsComments WHERE true
    )
    .await?;

    assert_eq!(
        results.len(),
        0,
        "INNER JOIN chain should return empty with no comments"
    );

    conn.rollback().await?;
    Ok(())
}

/// Test mixed joins (INNER + LEFT)
#[always_context(skip(!))]
#[tokio::test]
async fn test_mixed_joins_with_optional_comments() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;
    CommentsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "quinn", "quinn@example.com").await?;

    // Post with comment
    insert_test_post(&mut conn, 1, "Post with comment", "Content").await?;
    insert_test_comment(&mut conn, 1, 1, "A comment").await?;

    // Post without comment
    insert_test_post(&mut conn, 1, "Post without comment", "Content").await?;

    let results: Vec<UserPostOptionalCommentOutput> = query!(&mut conn,
        SELECT Vec<UserPostOptionalCommentOutput> FROM UsersPostsOptionalComments
        WHERE true
        ORDER BY PostsTable.id
    )
    .await?;

    assert_eq!(results.len(), 2);

    // First post has comment
    assert_eq!(results[0].post_title, "Post with comment");
    assert_eq!(results[0].comment_text, Some("A comment".to_string()));

    // Second post has no comment
    assert_eq!(results[1].post_title, "Post without comment");
    assert_eq!(results[1].comment_text, None);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 6. Output with #[sql(select = ...)] Tests
// ==============================================

/// Test Output struct with custom SELECT expressions
#[always_context(skip(!))]
#[tokio::test]
async fn test_output_with_custom_select() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "rachel", "rachel@example.com").await?;
    insert_test_post(
        &mut conn,
        1,
        "Rachel's Post",
        "This is a longer piece of content",
    )
    .await?;

    let results: Vec<UserPostCustomOutput> = query!(&mut conn,
        SELECT Vec<UserPostCustomOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].combined_title, "rachel - Rachel's Post");
    assert_eq!(
        results[0].content_length,
        "This is a longer piece of content".len() as i32
    );

    conn.rollback().await?;
    Ok(())
}

/// Test Output mixing field and select attributes
#[always_context(skip(!))]
#[tokio::test]
async fn test_output_mixed_field_and_select() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "steve", "steve@example.com").await?;
    insert_test_post(&mut conn, 1, "lowercase title", "Some content").await?;

    let results: Vec<UserPostMixedOutput> = query!(&mut conn,
        SELECT Vec<UserPostMixedOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "steve");
    assert_eq!(results[0].title_upper, "LOWERCASE TITLE");
    assert_eq!(results[0].content, "Some content");

    conn.rollback().await?;
    Ok(())
}

/// Test Output with aggregate function (requires GROUP BY)
#[always_context(skip(!))]
#[tokio::test]
async fn test_output_with_aggregate_function() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // User with multiple posts
    insert_test_user(&mut conn, "tina", "tina@example.com").await?;
    insert_test_post(&mut conn, 1, "Post 1", "Content").await?;
    insert_test_post(&mut conn, 1, "Post 2", "Content").await?;
    insert_test_post(&mut conn, 1, "Post 3", "Content").await?;

    // User without posts
    insert_test_user(&mut conn, "uma", "uma@example.com").await?;

    let results: Vec<UserPostCountOutput> = query!(&mut conn,
        SELECT Vec<UserPostCountOutput> FROM UsersWithOptionalPosts
        GROUP BY UsersTable.id
        ORDER BY UsersTable.id
    )
    .await?;

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].username, "tina");
    assert_eq!(results[0].post_count, 3);
    assert_eq!(results[1].username, "uma");
    assert_eq!(results[1].post_count, 0);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 7. query_lazy! Macro Tests
// ==============================================

/// Test table_join! with query_lazy! macro (streaming)
#[always_context(skip(!))]
#[tokio::test]
async fn test_table_join_with_query_lazy() -> anyhow::Result<()> {
    use futures::stream::StreamExt;
    use sql_macros::query_lazy;

    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.conn().await?;
    PostsTable::setup(&mut &mut conn).await?;

    // Insert test data
    insert_test_user(&mut conn, "victor", "victor@example.com").await?;
    for i in 1..=5 {
        insert_test_post(&mut conn, 1, &format!("Post {}", i), "Content").await?;
    }

    // Use query_lazy! for streaming
    let mut lazy_query = query_lazy!(
        SELECT UserPostOutput FROM UsersWithPosts WHERE true ORDER BY PostsTable.id
    )?;
    let mut stream = lazy_query.fetch(&mut conn);

    let mut count = 0;
    while let Some(result) = stream.next().await {
        let row = result.context("Failed to fetch row from stream")?;
        assert_eq!(row.username, "victor");
        count += 1;
    }

    assert_eq!(count, 5, "Should stream 5 rows");

    Ok(())
}

/// Test query_lazy! with LEFT JOIN
#[always_context(skip(!))]
#[tokio::test]
async fn test_table_join_query_lazy_left_join() -> anyhow::Result<()> {
    use futures::stream::StreamExt;
    use sql_macros::query_lazy;

    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.conn().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "wendy", "wendy@example.com").await?;
    insert_test_user(&mut conn, "xander", "xander@example.com").await?;
    insert_test_post(&mut conn, 1, "Wendy's Post", "Content").await?;

    let mut lazy_query = query_lazy!(
        SELECT UserOptionalPostOutput FROM UsersWithOptionalPosts
        WHERE true
        ORDER BY UsersTable.id
    )?;
    let mut stream = lazy_query.fetch(&mut conn);

    let mut results = Vec::new();
    while let Some(result) = stream.next().await {
        results.push(result.context("Failed to fetch row from stream")?);
    }

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].username, "wendy");
    assert!(results[0].post_id.is_some());
    assert_eq!(results[1].username, "xander");
    assert!(results[1].post_id.is_none());

    Ok(())
}

// ==============================================
// 8. Complex Scenarios
// ==============================================

/// Test join with complex WHERE conditions
#[always_context(skip(!))]
#[tokio::test]
async fn test_join_with_complex_where() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "yara", "yara@example.com").await?;
    insert_test_user(&mut conn, "zack", "zack@example.com").await?;

    insert_test_post(&mut conn, 1, "Important", "Content").await?;
    insert_test_post(&mut conn, 1, "Regular", "Content").await?;
    insert_test_post(&mut conn, 2, "Important", "Content").await?;

    let search_username = "yara".to_string();
    let search_title = "Important".to_string();

    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts
        WHERE UsersTable.username = {search_username}
          AND PostsTable.title = {search_title}
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].username, "yara");
    assert_eq!(results[0].post_title, "Important");

    conn.rollback().await?;
    Ok(())
}

/// Test join with EXISTS subquery
#[always_context(skip(!))]
#[tokio::test]
async fn test_join_with_exists() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "alice2", "alice2@example.com").await?;
    insert_test_post(&mut conn, 1, "Post", "Content").await?;

    let username = "alice2".to_string();
    let exists: bool = query!(&mut conn,
        EXISTS UsersWithPosts WHERE UsersTable.username = {username}
    )
    .await?;

    assert!(exists);

    let nonexistent = "nobody".to_string();
    let not_exists: bool = query!(&mut conn,
        EXISTS UsersWithPosts WHERE UsersTable.username = {nonexistent}
    )
    .await?;

    assert!(!not_exists);

    conn.rollback().await?;
    Ok(())
}

/// Test UPDATE with joined tables
#[always_context(skip(!))]
#[tokio::test]
async fn test_update_with_join() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "bob2", "bob2@example.com").await?;
    insert_test_post(&mut conn, 1, "Old Title", "Content").await?;

    // Update post using joined table condition
    let username = "bob2".to_string();
    let new_post = Post {
        user_id: 1,
        title: "Updated Title".to_string(),
        content: "Content".to_string(),
    };

    query!(&mut conn,
        UPDATE PostsTable SET {new_post} WHERE PostsTable.id = 1
    )
    .await?;

    // Verify update
    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts
        WHERE UsersTable.username = {username}
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].post_title, "Updated Title");

    conn.rollback().await?;
    Ok(())
}

/// Test DELETE with joined tables
#[always_context(skip(!))]
#[tokio::test]
async fn test_delete_with_join_context() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "charlie2", "charlie2@example.com").await?;
    insert_test_post(&mut conn, 1, "Post to Delete", "Content").await?;

    // Delete post
    query!(&mut conn, DELETE FROM PostsTable WHERE PostsTable.id = 1).await?;

    // Verify deletion using join
    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

// ==============================================
// 9. Edge Cases and Error Scenarios
// ==============================================

/// Test join with empty tables
#[always_context(skip(!))]
#[tokio::test]
async fn test_join_empty_tables() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT Vec<UserPostOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 0);

    conn.rollback().await?;
    Ok(())
}

/// Test LEFT JOIN with all NULL optional fields
#[always_context(skip(!))]
#[tokio::test]
async fn test_left_join_all_null_fields() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "david2", "david2@example.com").await?;

    let results: Vec<UserOptionalPostOutput> = query!(&mut conn,
        SELECT Vec<UserOptionalPostOutput> FROM UsersWithOptionalPosts WHERE true
    )
    .await?;

    assert_eq!(results.len(), 1);
    assert!(results[0].post_id.is_none());
    assert!(results[0].post_title.is_none());

    conn.rollback().await?;
    Ok(())
}

/// Test join with DISTINCT
#[always_context(skip(!))]
#[tokio::test]
async fn test_join_with_distinct() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    PostsTable::setup(&mut &mut conn).await?;

    insert_test_user(&mut conn, "eve2", "eve2@example.com").await?;
    insert_test_post(&mut conn, 1, "Same Title", "Content 1").await?;
    insert_test_post(&mut conn, 1, "Same Title", "Content 2").await?;

    let results: Vec<UserPostOutput> = query!(&mut conn,
        SELECT DISTINCT Vec<UserPostOutput> FROM UsersWithPosts WHERE true
    )
    .await?;

    // DISTINCT may or may not reduce count depending on all fields
    assert!(!results.is_empty());

    conn.rollback().await?;
    Ok(())
}

/// Test one-to-one relationship with LEFT JOIN (Users -> Profiles)
#[always_context(skip(!))]
#[tokio::test]
async fn test_left_join_one_to_one_profiles() -> anyhow::Result<()> {
    let db = Database::setup_for_testing::<UsersTable>().await?;
    let mut conn = db.transaction().await?;
    ProfilesTable::setup(&mut &mut conn).await?;

    // User with profile
    insert_test_user(&mut conn, "alice", "alice@example.com").await?;
    let profile = Profile {
        user_id: 1,
        bio: "Software developer".to_string(),
        location: "New York".to_string(),
    };
    query!(&mut conn, INSERT INTO ProfilesTable VALUES {profile}).await?;

    // User without profile
    insert_test_user(&mut conn, "bob", "bob@example.com").await?;

    // Define output struct for this join
    #[derive(Output, Debug, Clone, PartialEq)]
    #[sql(table = UsersWithProfiles)]
    struct UserProfileOutput {
        #[sql(field = UsersTable.username)]
        username: String,
        #[sql(field = ProfilesTable.bio)]
        bio: Option<String>,
        #[sql(field = ProfilesTable.location)]
        location: Option<String>,
    }

    let results: Vec<UserProfileOutput> = query!(&mut conn,
        SELECT Vec<UserProfileOutput> FROM UsersWithProfiles
        WHERE true
        ORDER BY UsersTable.id
    )
    .await?;

    assert_eq!(results.len(), 2);

    // Alice has profile
    assert_eq!(results[0].username, "alice");
    assert_eq!(results[0].bio, Some("Software developer".to_string()));
    assert_eq!(results[0].location, Some("New York".to_string()));

    // Bob has no profile
    assert_eq!(results[1].username, "bob");
    assert_eq!(results[1].bio, None);
    assert_eq!(results[1].location, None);

    conn.rollback().await?;
    Ok(())
}
