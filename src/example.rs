fn init_example() -> anyhow::Result<()> {
    //Module name is used as a schema in database
    setup_sql::<DatabaseSetup>("module_name").await?;

    Ok(())
}
#[derive(DatabaseSetup)]
struct DatabaseSetup {
    //Show error when table is used in query but not in database setup
    //Derive also implements `UsedInDatabase` Trait for types inside
    t1: ExampleTableStructure,
}

#[derive(Serialize, Deserialize)]
enum ExampleEnum {
    A,
    B,
    C,
}

//Implements also SqlDbOutput Trait
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
#[derive(SqlDbOutput)]
#[sql(supports=ExampleTableStructure)]
struct ExampleOutput {
    id: i64,
    #[sql(database_field = field1)]
    nnnn: String,
}

fn get_data_example() -> anyhow::Result<()> {
    let whatever = 15;

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
