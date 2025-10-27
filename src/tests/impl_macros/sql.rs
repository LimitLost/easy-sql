use easy_macros::macros::always_context;

use super::{DatabaseInternalDefault, TestDriver};
use crate::{Connection, Expr, QueryBuilder, Table, TableJoin, never::never_any};

#[allow(dead_code)]
struct ExampleTable {
    id: i64,
    field0: String,
    field1: String,
    field2: i32,
    field3: i64,
    field4: i16,
}

#[always_context]
impl Table<TestDriver> for ExampleTable {
    fn table_name() -> &'static str {
        "example_table"
    }

    fn primary_keys() -> Vec<&'static str> {
        vec!["id"]
    }

    fn table_joins(_builder: &mut QueryBuilder<'_, TestDriver>) -> Vec<TableJoin> {
        vec![]
    }
}

// Compilation Lifetime test
#[always_context]
#[no_context]
async fn _test() -> anyhow::Result<()> {
    use crate::{Table, WhereClause};

    let mut fake_conn = never_any::<Connection<TestDriver, DatabaseInternalDefault>>();

    let test_arg1 = 5;
    let test_arg2 = "Hello".to_string();

    ExampleTable::update(
        &mut fake_conn,
        (vec![], |b: &mut QueryBuilder<TestDriver>| {
            // Fully safe because values live until the end of ExampleTable::update invocation
            unsafe {
                b.bind(&test_arg1)?;
                b.bind(&test_arg2)?;
            }
            Ok(())
        }),
        |b: &mut QueryBuilder<TestDriver>| {
            // Fully safe because values live until the end of ExampleTable::update invocation
            unsafe {
                b.bind(&test_arg1)?;
                let _ = || &test_arg2;
                b.bind(&test_arg2)?;
                /* || {
                    let should_fail = true;
                    &should_fail
                }; */
                let should_fail = true;
                b.bind(&should_fail)?;
            }
            Ok(WhereClause {
                conditions: Expr::Eq(
                    Box::new(Expr::Column("id".to_string())),
                    Box::new(Expr::Value),
                ),
            })
        },
    )
    .await?;

    Ok(())
}
