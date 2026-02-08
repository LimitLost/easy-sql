use easy_macros::{add_code, always_context};

use crate as easy_sql;

#[always_context(skip(!))]
#[no_context]
#[add_code(after ={
    main().await?;
    Ok(())
})]
#[tokio::test]
#[docify::export_content]
#[allow(dead_code)]
async fn mini_demo() -> anyhow::Result<()> {
    use easy_sql::sqlite::Database;
    use easy_sql::{DatabaseSetup, Insert, Output, Table, query};

    // DatabaseSetup lets you group tables into a single setup call.
    #[derive(DatabaseSetup)]
    struct PartOfDatabase {
        users: UserTable,
    }

    #[derive(Table)]
    struct UserTable {
        #[sql(primary_key)]
        #[sql(auto_increment)]
        id: i32,
        email: String,
        active: bool,
    }

    #[derive(Insert)]
    #[sql(table = UserTable)]
    // Required to make sure that no fields are potentially ignored
    #[sql(default = id)]
    struct NewUser {
        email: String,
        active: bool,
    }

    #[derive(Output)]
    #[sql(table = UserTable)]
    struct UserRow {
        id: i32,
        #[sql(select = email || " (active = " || active || ")")]
        email_label: String,
        active: bool,
    }
    async fn main() -> anyhow::Result<()> {
        let db = Database::setup::<PartOfDatabase>("app.sqlite").await?;
        let mut conn = db.conn().await?;

        let data = NewUser {
            email: "sam@example.com".to_string(),
            active: true,
        };
        query!(&mut conn, INSERT INTO UserTable VALUES {data}).await?;

        let new_email = "sammy@example.com";
        query!(&mut conn,
            UPDATE UserTable SET active = false, email = {new_email} WHERE UserTable.email = "sam@example.com"
        )
        .await?;

        let row: UserRow = query!(&mut conn,
            SELECT UserRow FROM UserTable WHERE email = {new_email}
        )
        .await?;

        println!("{} {}", row.id, row.email_label);
        Ok(())
    }
}
