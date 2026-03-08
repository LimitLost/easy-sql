use std::fmt::Display;
use std::ops::Bound;

use crate::{ToDefault, drivers::postgres::Postgres};
use sqlx::postgres::types::{
    PgBox, PgCircle, PgCube, PgHstore, PgLSeg, PgLine, PgMoney, PgPath, PgPoint, PgPolygon, PgRange,
};

fn to_default_postgres<T: ToDefault<Postgres>>(value: T) -> String {
    value.to_default_failable().unwrap()
}

fn assert_owned_ref_sql_eq<T>(owned: T, reference: T, expected: &str)
where
    T: ToDefault<Postgres>,
    for<'a> &'a T: ToDefault<Postgres>,
{
    let owned_sql = to_default_postgres(owned);
    let reference_sql = to_default_postgres(&reference);
    assert_eq!(owned_sql, expected);
    assert_eq!(owned_sql, reference_sql);
}

fn assert_owned_ref_sql_suffix<T>(owned: T, reference: T, suffix: &str)
where
    T: ToDefault<Postgres>,
    for<'a> &'a T: ToDefault<Postgres>,
{
    let owned_sql = to_default_postgres(owned);
    let reference_sql = to_default_postgres(&reference);
    assert!(owned_sql.ends_with(suffix));
    assert_eq!(owned_sql, reference_sql);
}

#[derive(serde::Serialize, Clone)]
struct JsonPayload {
    label: &'static str,
    count: i32,
}

#[test]
fn postgres_to_default_extended_json_generic() {
    // Generic wrappers with local payload types are kept as focused unit checks.
    let json_owned = sqlx::types::Json(JsonPayload {
        label: "x",
        count: 7,
    });
    let json_ref = sqlx::types::Json(JsonPayload {
        label: "x",
        count: 7,
    });

    assert_owned_ref_sql_eq(
        json_owned,
        json_ref,
        "'{\"label\":\"x\",\"count\":7}'::jsonb",
    );
}

#[derive(Debug, Clone)]
struct DisplayPayload(i32);

impl Display for DisplayPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "payload-{}", self.0)
    }
}

#[test]
fn postgres_to_default_extended_text_generic() {
    let text_owned = sqlx::types::Text(DisplayPayload(9));
    let text_ref = sqlx::types::Text(DisplayPayload(9));

    assert_owned_ref_sql_eq(text_owned, text_ref, "'payload-9'");
}

#[test]
fn postgres_to_default_extended_type_rendering_smoke() {
    let mut hstore_map = std::collections::BTreeMap::new();
    hstore_map.insert("k".to_string(), Some("v".to_string()));
    let hstore = PgHstore(hstore_map);
    assert_owned_ref_sql_suffix(hstore.clone(), hstore, "::hstore");

    assert_owned_ref_sql_suffix(PgMoney(12_345), PgMoney(12_345), "::money");

    let cube = PgCube::Point(1.25);
    assert_owned_ref_sql_suffix(cube.clone(), cube, "::cube");

    let box_value = PgBox {
        upper_right_x: 5.0,
        upper_right_y: 6.0,
        lower_left_x: 1.0,
        lower_left_y: 2.0,
    };
    assert_owned_ref_sql_suffix(box_value.clone(), box_value, "::box");

    let circle = PgCircle {
        x: 1.0,
        y: 2.0,
        radius: 3.0,
    };
    assert_owned_ref_sql_suffix(circle.clone(), circle, "::circle");

    let segment = PgLSeg {
        start_x: 1.0,
        start_y: 2.0,
        end_x: 3.0,
        end_y: 4.0,
    };
    assert_owned_ref_sql_suffix(segment.clone(), segment, "::lseg");

    let line = PgLine {
        a: 1.0,
        b: 2.0,
        c: 3.0,
    };
    assert_owned_ref_sql_suffix(line.clone(), line, "::line");

    let path = PgPath {
        closed: true,
        points: vec![PgPoint { x: 1.0, y: 2.0 }, PgPoint { x: 3.0, y: 4.0 }],
    };
    assert_owned_ref_sql_suffix(path.clone(), path, "::path");

    let point = PgPoint { x: 9.0, y: 8.0 };
    assert_owned_ref_sql_suffix(point.clone(), point, "::point");

    let polygon = PgPolygon {
        points: vec![
            PgPoint { x: 0.0, y: 0.0 },
            PgPoint { x: 1.0, y: 0.0 },
            PgPoint { x: 1.0, y: 1.0 },
            PgPoint { x: 0.0, y: 1.0 },
        ],
    };
    assert_owned_ref_sql_suffix(polygon.clone(), polygon, "::polygon");

    let range = PgRange {
        start: Bound::Included(1_i32),
        end: Bound::Excluded(3_i32),
    };
    assert_owned_ref_sql_eq(range, range, "'[\"1\",\"3\")'");

    #[cfg(feature = "time")]
    {
        let timetz = sqlx::postgres::types::PgTimeTz {
            time: sqlx::types::time::Time::from_hms(3, 4, 5).unwrap(),
            offset: sqlx::types::time::UtcOffset::UTC,
        };
        let timetz_ref = sqlx::postgres::types::PgTimeTz {
            time: sqlx::types::time::Time::from_hms(3, 4, 5).unwrap(),
            offset: sqlx::types::time::UtcOffset::UTC,
        };
        assert_owned_ref_sql_suffix(timetz, timetz_ref, "::timetz");
    }
}

#[test]
fn postgres_to_default_extended_escaping_and_range_edges() {
    let mut escaped_hstore_map = std::collections::BTreeMap::new();
    escaped_hstore_map.insert("k\"\\x".to_string(), Some("v\"\\y".to_string()));
    let hstore_sql = to_default_postgres(PgHstore(escaped_hstore_map));
    assert!(hstore_sql.ends_with("::hstore"));
    assert!(hstore_sql.contains("\\\\"));
    assert!(hstore_sql.contains("\\\""));

    let json_sql = to_default_postgres(sqlx::types::Json(serde_json::json!({"k": "v\"\\z"})));
    assert!(json_sql.ends_with("::jsonb"));
    assert!(json_sql.contains("\\\\"));
    assert!(json_sql.contains("\\\""));

    let text_sql = to_default_postgres(sqlx::types::Text("a'b\\c".to_string()));
    assert!(text_sql.contains("''"));
    assert!(text_sql.contains("\\"));

    let unbounded_range_sql = to_default_postgres(PgRange::<i32> {
        start: Bound::Unbounded,
        end: Bound::Unbounded,
    });
    assert_eq!(unbounded_range_sql, "'(,)'");

    let escaped_bound_range_sql = to_default_postgres(PgRange {
        start: Bound::Included("a\"\\b".to_string()),
        end: Bound::Unbounded,
    });
    assert!(escaped_bound_range_sql.contains("\"a"));
    assert!(escaped_bound_range_sql.contains("\\\\"));
    assert!(escaped_bound_range_sql.contains("\\\""));
}

#[cfg(feature = "bstr")]
#[test]
fn postgres_to_default_extended_bstring() {
    let bytes = vec![0_u8, 1_u8, 2_u8, 255_u8];
    let owned = sqlx::types::BString::from(bytes.clone());
    let by_ref = sqlx::types::BString::from(bytes);

    assert_owned_ref_sql_eq(owned, by_ref, "'\\x000102ff'::bytea");
}
