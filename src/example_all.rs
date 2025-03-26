lazy_static! {
    static ref DB_BASE: Mutex<Option<Database>> = Mutex::new(None);
    static ref DB: Database = DB_BASE.lock().take().unwrap();
}

#[always_context]
fn init_example() -> anyhow::Result<()> {
    //Module name is used as a schema in database
    let db = Database::setup::<DatabaseSetup>("module_name").await?;
    DB_BASE.lock().replace(db);

    Ok(())
}
#[derive(DatabaseSetup)]
struct DatabaseSetupMain {
    //Show error when table is used in query but not in database setup
    //Derive also implements `UsedInDatabase` Trait for types inside
    t1: ExampleTableStructure,

    //TODO Someday in the future
    #[database_setup(inner)]
    t2: DatabaseSetup2,
}

#[derive(Serialize, Deserialize)]
enum ExampleEnum {
    A,
    B,
    C,
}

//Implements also SqlOutput, SqlInsert, SqlUpdate Traits
#[derive(SqlTable)]
//Name can also be auto generated
#[sql(table_name = example_table1)]
struct ExampleTableStructure {
    #[sql(auto_increment)]
    id: i64,
    field1: String,
    //Has Type in Postgresql it is send as Postgres type,
    //converted to binary in Sqlite
    field2: Vec<i64>,
    //Needs to implement Serialize and Deserialize
    //Converted to binary in Postgresql and Sqlite
    field3: ExampleEnum,
}

//Do static checks to see if table has those fields
//Can Implement DoesNotNeedGroupBy Trait
#[derive(SqlOutput)]
#[sql(supports=ExampleTableStructure)]
struct ExampleOutput {
    id: i64,
    #[sql(database_field = field1)]
    nnnn: String,
}
#[derive(SqlInsert)]
#[sql(supports=ExampleTableStructure)]
struct ExampleInsert {
    field1: String,
    field2: Vec<i64>,
    field3: ExampleEnum,
}
#[derive(SqlUpdate)]
#[sql(supports=ExampleTableStructure)]
struct ExampleUpdate {
    field1: String,
    field2: Vec<i64>,
}

#[always_context]
async fn get_data_example() -> anyhow::Result<()> {
    let conn = DB.conn().await?;

    let ins = ExampleInsert {
        field1: "test".to_string(),
        field2: vec![1, 2, 3],
        field3: ExampleEnum::A,
    };

    let whatever = 15;

    //Insert
    //Type inserted needs to implement SqlInsert, auto implement SqlInsert for Vec<T>
    ExampleTableStructure::insert(&conn, ins).await?;
    ExampleTableStructure::insert(&conn, vec![ins]).await?;

    //Insert with returning
    let r: ExampleOutput = ExampleTableStructure::insert_returning(&conn, ins).await?;
    let r2: Vec<ExampleOutput> = ExampleTableStructure::insert_returning(&conn, vec![ins]).await?;

    //Update
    let update = ExampleUpdate {
        field1: "test2".to_string(),
        field2: vec![1, 2, 3],
    };
    //TODO Put syntax_check:fn(Table) inside of Where used only for type checking inside of Where type
    ExampleTableStructure::update(&conn, update, sql_where!(id == 15)).await?;

    //Delete
    ExampleTableStructure::delete(&conn, sql_where!(id == { whatever })).await?;

    //Select
    //Add synonymous function select (only name was changed)
    let get1: ExampleOutput = ExampleTableStructure::get(&conn, Where::all()).await?;
    let get2: Vec<ExampleOutput> = ExampleTableStructure::select(&conn, Where::all()).await?;
    //TODO lazy_get and lazy_select (needing &mut conn)

    //Macros:
    //sql! - for sql statements (doesn't matter if eager or lazy loading)
    //sql_lazy! - for lazy loading
    //sql_eager! - for eager loading

    //Commands for sql select
    {
        let mut result: Vec<ExampleTableStructure> = Vec::new();
        let w = whatever as i64;
        for row in sql_lazy!(select ExampleTableStructure from ExampleTableStructure where id > {w})
            .await?
        {
            result.push(row);
        }
        result
    };
    //Simple select
    sql!(select ExampleOutput from ExampleTableStructure where id > {whatever} limit 1).await?;

    let values = vec![
        ExampleTableStructure {
            id: 1,
            field1: "test".to_string(),
            field2: vec![1, 2, 3],
            field3: ExampleEnum::A,
        },
        ExampleTableStructure {
            id: 2,
            field1: "test".to_string(),
            field2: vec![1, 2, 3],
            field3: ExampleEnum::A,
        },
    ];

    //Insert
    sql!(insert into ExampleTableStructure values {values}).await?;

    //Insert with returning
    sql_simple(sql!(insert into ExampleTableStructure values {values} returning ExampleOutput))
        .await?;

    //Update
    sql_simple(sql!(update ExampleTableStructure set field1 = "test2" where id = {whatever}))
        .await?;

    //Delete
    sql_simple(sql!(delete from ExampleTableStructure where id > {whatever})).await?;
    sql(|| {
        sql!(delete from ExampleTableStructure where id > {whatever});
    })
    .await?;

    Ok(())
}
